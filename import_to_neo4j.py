import os
import argparse
import subprocess
import json
import logging
from neo4j import GraphDatabase

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')

def run_command(command, cwd=None):
    """Runs a shell command and checks for errors."""
    logging.info(f"Running command: {command}")
    try:
        result = subprocess.run(
            command, 
            shell=True, 
            cwd=cwd, 
            check=True, 
            stdout=subprocess.PIPE, 
            stderr=subprocess.PIPE,
            text=True
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        logging.error(f"Command failed: {e.stderr}")
        raise

def build_move_project(project_path):
    """Attempts to build the Move project to generate bytecode."""
    move_toml_path = os.path.join(project_path, "Move.toml")
    
    # If explicit Move project
    if os.path.exists(move_toml_path):
        logging.info(f"Found Move.toml in {project_path}. Building...")
        try:
            # Check if sui is installed
            subprocess.run(["sui", "--version"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            # Build
            run_command("sui move build", cwd=project_path)
            logging.info("Build successful.")
        except FileNotFoundError:
             logging.error("Error: 'sui' command not found. Please install Sui CLI.")
             raise
        except subprocess.CalledProcessError as e:
            logging.error(f"Build failed: {e.stderr}")
            raise RuntimeError("Project build failed. Cannot proceed without bytecode.")
    else:
        logging.warning(f"No Move.toml found in {project_path}. Assuming bytecode files are manually placed.")

def find_bytecode_dir(project_path):
    """Finds the directory containing .mv files to scan."""
    # After build, artifacts are usually in build/
    build_dir = os.path.join(project_path, "build")
    
    if os.path.exists(build_dir):
        # Recursive scanner will handle subdirectories in build/
        return build_dir
    
    # Check if user passed a raw bytecode dir
    return project_path

def import_to_neo4j(uri, user, password, graph_data):
    """Imports graph data into Neo4j."""
    driver = GraphDatabase.driver(uri, auth=(user, password))
    
    def create_nodes_and_edges(tx, data):
        # 1. Clear database (Optional: use with caution)
        # tx.run("MATCH (n) DETACH DELETE n") 

        def _process_module(tx, node):
            props = {k: v for k, v in node.items() if k != "type"}
            query = """
            MERGE (n:Module {id: $id})
            ON CREATE SET n += $props
            ON MATCH SET n += $props
            """
            tx.run(query, id=props['id'], props=props)

        def _process_struct(tx, node):
            props = {k: v for k, v in node.items() if k != "type"}
            # Read 'source' from JSON (mapped from Rust 'source' field)
            source_code = node.get("source", "")
            props['source_code'] = source_code
            # Remove 'source' key if present to avoid duplication if we want clean props, 
            # but 'source' came from node which we filtered 'type' out of.
            # If props has 'source', we can keep it or remove it. 
            # We explicitly set 'source_code'.
            if 'source' in props:
                del props['source']

            query = """
            MERGE (n:Struct {id: $id})
            ON CREATE SET n += $props
            ON MATCH SET n += $props
            """
            tx.run(query, id=props['id'], props=props)

        def _process_function(tx, node):
            props = {k: v for k, v in node.items() if k != "type"}
            source_code = node.get("source", "")
            props['source_code'] = source_code
            if 'source' in props:
                del props['source']

            # Create derived property node_description
            name = props.get("name", "")
            module_id = props.get("module_id", "")
            # "Function " + name + " defined in " + module + ". Code: " + source_code
            node_description = f"Function {name} defined in {module_id}. Code: {source_code}"
            props['node_description'] = node_description

            query = """
            MERGE (n:Function {id: $id})
            ON CREATE SET n += $props
            ON MATCH SET n += $props
            """
            tx.run(query, id=props['id'], props=props)

        # 2. Create Nodes
        logging.info("Creating Nodes...")
        for node in data.get("nodes", []):
            node_type = node.get("type", "Unknown")
            if node_type == "Function":
                _process_function(tx, node)
            elif node_type == "Struct":
                _process_struct(tx, node)
            elif node_type == "Module":
                _process_module(tx, node)
            else:
                # Generic fallback for other types
                props = {k: v for k, v in node.items() if k != "type"}
                query = f"MERGE (n:{node_type} {{id: $props.id}}) SET n += $props"
                tx.run(query, props=props)

        # 3. Create Edges
        logging.info("Creating Edges...")
        for edge in data.get("edges", []):
            edge_type = edge.get("type", "RELATED_TO")
            source_id = edge.get("from")
            target_id = edge.get("to")
            
            query = f"""
            MATCH (a {{id: $source_id}})
            MATCH (b {{id: $target_id}})
            MERGE (a)-[r:{edge_type}]->(b)
            """
            tx.run(query, source_id=source_id, target_id=target_id)

    
    with driver.session() as session:
        session.execute_write(create_nodes_and_edges, graph_data)
    
    driver.close()
    logging.info("Import completed successfully.")

def main():
    parser = argparse.ArgumentParser(description="MoveScanner Automation Script")
    parser.add_argument("project_path", help="Path to the Move project directory")
    parser.add_argument("--scanner-bin", default="./target/debug/MoveScanner", help="Path to MoveScanner binary")
    parser.add_argument("--neo4j-uri", default="bolt://localhost:7687", help="Neo4j URI")
    parser.add_argument("--neo4j-user", default="neo4j", help="Neo4j User")
    parser.add_argument("--neo4j-pass", default="password", help="Neo4j Password")
    parser.add_argument("--output-dir", default="./res", help="Output directory for JSON")
    
    args = parser.parse_args()

    # 1. Build Project
    build_move_project(args.project_path)
    
    # 2. Determine Bytecode Path
    bytecode_path = find_bytecode_dir(args.project_path)
    logging.info(f"Scanning bytecode in: {bytecode_path}")

    # 3. Run Scanner
    output_json = os.path.join(args.output_dir, "output.json")
    graph_json = os.path.join(args.output_dir, "output_graph.json") # Scanner auto-appends _graph.json
    
    # Ensure output dir exists
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Use cargo run or binary
    if args.scanner_bin.startswith("cargo"):
        cmd = f"{args.scanner_bin} -- -p {bytecode_path} -s {args.project_path} --skip-build -o {output_json}"
    else:
        cmd = f"{args.scanner_bin} -p {bytecode_path} -s {args.project_path} --skip-build -o {output_json}"
        
    run_command(cmd)

    # 4. Read Graph JSON
    if not os.path.exists(graph_json):
        logging.error(f"Graph file not found: {graph_json}")
        return

    with open(graph_json, 'r') as f:
        graph_data = json.load(f)

    # 5. Import to Neo4j
    try:
        import_to_neo4j(args.neo4j_uri, args.neo4j_user, args.neo4j_pass, graph_data)
    except Exception as e:
        logging.error(f"Neo4j Import Failed: {e}")

if __name__ == "__main__":
    main()
