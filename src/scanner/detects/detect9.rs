// recursive_function_call

use crate::{
    move_ir::{generate_bytecode::StacklessBytecodeGenerator, packages::Packages},
    scanner::{detectors::AbstractDetector, result::*},
};
use move_model::{
    model::{FunId, QualifiedId},
    symbol::SymbolPool,
};
use petgraph::{
    algo::is_cyclic_directed,
    prelude::{Direction, NodeIndex},
    Graph,
};

pub struct Detector9<'a> {
    packages: &'a Packages<'a>,
    content: DetectContent,
}

impl<'a> AbstractDetector<'a> for Detector9<'a> {
    fn new(packages: &'a Packages<'a>) -> Self {
        Self {
            packages,
            content: DetectContent::new(Severity::Major, DetectKind::RecursiveFunctionCall),
        }
    }
    fn run(&mut self) -> &DetectContent {
        for (mname, &ref stbgr) in self.packages.get_all_stbgr().iter() {
            self.content.result.insert(mname.to_string(), Vec::new());
            if let Some(res) = self.detect_recursive_function_call(stbgr) {
                for fun_name in res.iter() {
                    self.content
                        .result
                        .get_mut(mname)
                        .unwrap()
                        .push(fun_name.to_string());
                }
            }
        }
        &self.content
    }
}

impl<'a> Detector9<'a> {
    pub fn detect_recursive_function_call(
        &mut self,
        stbgr: &StacklessBytecodeGenerator,
    ) -> Option<Vec<String>> {
        let symbol_pool = &stbgr.symbol_pool;
        // 1. 检查当前模块的函数调用图中是否存在环,若不存在,则说明无递归调用,直接跳过检测
        let has_cycle = is_cyclic_directed(&stbgr.call_graph);
        if !has_cycle {
            return None;
        }
        // 2. 克隆函数调用图，因为后续要删除边,避免影响到其他检测器
        let call_graph = &mut stbgr.call_graph.clone();
        let nodes = &stbgr.func_to_node;
        let mut cycle_paths: Vec<Vec<NodeIndex>> = Vec::new();
        // 3. 函数调用图的节点是函数,遍历所有节点
        nodes.iter().for_each(|(_qid, nodeid)| {
            // 在图中寻找以当前节点为起始点的所有环的路径
            self.find_cycles(call_graph, *nodeid, &mut cycle_paths);
        });
        // 4. cycle_paths 中存储的是节点id,将其转换为具体的函数名
        let cycle_paths_str = self.convert_cycle_paths(&cycle_paths, &call_graph, symbol_pool);
        return Some(cycle_paths_str);
    }

    // 寻找以某个节点为起始点时,图中的所有环的路径
    fn find_cycles(
        &mut self,
        graph: &mut Graph<QualifiedId<FunId>, ()>,
        start_node: petgraph::graph::NodeIndex,
        cycle_paths: &mut Vec<Vec<NodeIndex>>,
    ) {
        // 尝试寻找环
        let cycle_path = self.find_cycle(graph, start_node);
        // 若没有找到环,则直接返回
        if cycle_path.is_empty() {
            return;
        }
        // 若找到环:
        // 1. 将找到的环保存到 cycle_paths 中
        cycle_paths.push(cycle_path.clone());
        // 2. 在图中删除已找到的环的边
        self.remove_cycle(graph, &cycle_path);
        // 3. 递归调用,尝试寻找以当前节点为起始点时,其余的环
        self.find_cycles(graph, start_node, cycle_paths);
    }

    // 寻找以某个节点为起始点时,图中的一个环路径
    fn find_cycle(
        &mut self,
        graph: &Graph<QualifiedId<FunId>, ()>,
        start_node: petgraph::graph::NodeIndex,
    ) -> Vec<NodeIndex> {
        // 记录节点是否被访问
        let mut visited = vec![false; graph.node_count()];
        // 保存已访问过的节点, e.g. stack=[0,1,2,3,4,2] 说明存在环 2->3->4->2
        let mut stack: Vec<NodeIndex> = Vec::new();

        // 标志是否找到环
        let mut find_cycle = false;
        // 通过深度遍历寻找环
        self.dfs(
            &graph,
            start_node,
            &mut visited,
            &mut stack,
            &mut find_cycle,
        );
        // 通过 stack 获取环路径, e.g. cycle_path=[2,3,4,2]
        let mut cycle_path = Vec::new();

        // 如果没有找到环,则返回空路径
        if !find_cycle {
            // println!("No cycle found.");
            return cycle_path;
        }
        // 开始构建 circle_path,栈顶元素是循环的起点
        let circle_start_node = stack.pop().unwrap();
        cycle_path.push(circle_start_node.clone());
        while let Some(x) = stack.pop() {
            cycle_path.insert(0, x);
            // 找到某个和 circle_start_node 相等的元素时，表明循环结束
            if x == circle_start_node {
                break;
            }
        }
        // print!("cycle: [ ");
        // for element in &cycle_path {
        //     print!("{} ", element.index());
        // }
        // println!("]");
        cycle_path
    }

    // 删除已发现的环的边
    fn remove_cycle(
        &mut self,
        graph: &mut Graph<QualifiedId<FunId>, ()>,
        cycle_path: &Vec<NodeIndex>,
    ) {
        // 假设 cycle_path=[2,3,4,2]，表示2调用3，3再调用4，4调用2，此时删除 2->3、3->4 和 4->2 三条边
        for i in 0..(cycle_path.len() - 1) {
            let from = cycle_path[i];
            let to = cycle_path[i + 1];
            if let Some(edge_id) = graph.find_edge(from, to) {
                graph.remove_edge(edge_id);
            }
        }
    }
    // 深度遍历寻找环
    fn dfs(
        &mut self,
        graph: &Graph<QualifiedId<FunId>, ()>,
        node: NodeIndex,
        visited: &mut Vec<bool>,
        stack: &mut Vec<NodeIndex>,
        find_cycle: &mut bool,
    ) {
        // TIP1: 已经找到环,则直接返回,不再遍历当前节点分支
        // 否则若出现 0->1 0->2 2->0 时,若先走了 0->2 2->0,此时已经发现循环,但会继续走 0->1,并将其压入栈中
        // 由于已经发现了循环,因此 1 不会被弹出,此时 stack=[0,2,0,1],因此再已经发现分支时应该终止迭代
        if *find_cycle {
            return;
        }
        // 记录路径
        stack.push(node.clone());
        // 标记当前节点为已访问
        visited[node.index()] = true;
        // 遍历当前节点的所有子节点
        for child_node in graph.neighbors_directed(node, Direction::Outgoing) {
            // 若未访问过,则深度遍历子节点
            if !visited[child_node.index()] {
                self.dfs(graph, child_node, visited, stack, find_cycle);
            } else {
                // 若已访问过，则表示环的起点和终点重合,找到环。
                stack.push(child_node.clone());
                *find_cycle = true;
                return;
            }
        }

        // TIP2: 当前节点的所有分支中均未找到环,说明当前节点不在环中,将当前节点从 stack 中弹出
        // 否则若出现 0->1 0->2 2->0 时,若先走了 0->1 分支,此时已经将 1 压入栈,此时 stack=[0,1,2,0],因此需要把 2 弹出
        if !*find_cycle {
            stack.pop();
            // TIP3: 当前节点的所有分支中均未找到环,则将该节点标记为未访问(对于新的分支而言,它确实是未访问的)
            // 否则若出现 0->1 1->2 0->2 时,先访问 0->1,1->2,将0,1,2都置为已访问, 随后走0->2时,发现 2 已访问,错误的认为存在环
            // 因此需要在确保当前分支不存在环时将 1 和 2 都置为未访问.
            visited[node.index()] = false;
        }
    }

    fn convert_cycle_paths(
        &self,
        cycle_paths: &Vec<Vec<NodeIndex>>,
        graph: &Graph<QualifiedId<FunId>, ()>,
        symbol_pool: &SymbolPool,
    ) -> Vec<String> {
        let mut ret: Vec<String> = vec![];
        for cycle_path in cycle_paths {
            let cycle_path_str = format!(
                "({})",
                cycle_path
                    .into_iter()
                    .map(|nodeid| {
                        return self.get_function_name(nodeid, graph, symbol_pool).unwrap();
                    })
                    .collect::<Vec<String>>()
                    .join("->")
            );
            ret.push(cycle_path_str);
        }
        ret
    }

    fn get_function_name(
        &self,
        nodeid: &NodeIndex,
        graph: &Graph<QualifiedId<FunId>, ()>,
        symbol_pool: &SymbolPool,
    ) -> Option<String> {
        if let Some(qualified_id) = graph.node_weight(*nodeid) {
            let fid = &qualified_id.id;
            let fname = fid.symbol().display(symbol_pool).to_string();
            Some(fname)
        } else {
            None
        }
    }
}
