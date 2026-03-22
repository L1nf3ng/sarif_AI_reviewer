use tree_sitter::Parser;
use tree_sitter_python::LANGUAGE;

// 官方关于rust版tree-sitter的介绍文档
// https://github.com/tree-sitter/tree-sitter/blob/master/docs

pub fn parse_source_string() {
    let code = r#"
        import re

        class A:
            a = 3

            def __init__(self, t):
                self.a = t

            def __hash__(self):
                return hash(self)

        def double(x):
            return x * 2
    "#;

    let mut parser = Parser::new();
    let lang = LANGUAGE;
    parser
        .set_language(&lang.into())
        .expect("初始化失败，请检查配置");
    let tree = parser.parse(code, None).unwrap();
    let mut cursor = tree.walk();

    // 使用BFS策略。因为这里的Vec是后进先出的，所以选择了逆序，第一个子节点放在栈尾，pop后先出来。
    let mut node_stack = Vec::new();
    node_stack.push((tree.root_node(), 0));

    // while let Some((node, depth)) = node_stack.first().copied() {
    while let Some((node, depth)) = node_stack.pop() {
        // node_stack.remove(0);

        // 缩进
        let indent = "  ".repeat(depth);
        // 节点类型名称
        let type_name = node.kind();
        // 节点范围（起始行、起始列、结束行、结束列）
        let range = node.range();
        // 提取代码片段
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let snippet = &code[start_byte..end_byte];

        // 打印节点信息
        println!(
            "{}[{}] 行:{}-{} 列:{}-{} | 代码: {:?}",
            indent,
            type_name,
            range.start_point.row + 1,
            range.end_point.row + 1,
            range.start_point.column,
            range.end_point.column,
            snippet
        );

        let mut children: Vec<_> = node.children(&mut cursor).collect();
        children.reverse();
        for child in children {
            node_stack.push((child, depth + 1));
        }
    }
}

pub fn parse_source_file() {
    
    
}
