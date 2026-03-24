use std::error::Error;

use tokio::fs;
use std::collections::HashMap;

use tree_sitter::{Parser, Node};
use tree_sitter_python::LANGUAGE as PyLanguage;
use tree_sitter_java::LANGUAGE as JavaLanguage;

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
    let lang = PyLanguage;
    parser
        .set_language(&lang.into())
        .expect("初始化失败，请检查配置");
    let tree = parser.parse(code, None).unwrap();
    let mut cursor = tree.walk();

    // 使用BFS策略。因为这里的Vec是后进先出的，所以选择了逆序，第一个子节点放在栈尾，pop后先出来。
    let mut node_stack = Vec::new();
    node_stack.push((tree.root_node(), 0));

    while let Some((node, depth)) = node_stack.first().copied() {
    // while let Some((node, depth)) = node_stack.pop() {
        node_stack.remove(0);

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

        let children: Vec<Node> = node.children(&mut cursor).collect();
        // children.reverse();
        for child in children {
            node_stack.push((child, depth + 1));
        }
    }
}

pub async fn parse_source_file(filepath: &str, language: &str, line_numbers: Vec<usize>) -> Result<Vec<String>, Box<dyn Error>>{
    let source_code = fs::read_to_string(filepath).await?;
    let mut parser = match language{
        "python"=> {    
            let mut parser = Parser::new();
            let lang = PyLanguage;
            parser.set_language(&lang.into()).expect("初始化失败，请检查配置");
            parser
        },
        _=> {
            let mut parser = Parser::new();
            let lang = JavaLanguage;
            parser.set_language(&lang.into()).expect("初始化失败，请检查配置");
            parser
        }
    };
    
    // todo! finish parse content
    let tree = parser.parse(&source_code, None).unwrap();
    // 每一个行号都找到它的最小作用域（即所在函数），然后返回代码段。
    let mut cursor = tree.walk();

    // 使用BFS策略。因为这里的Vec是后进先出的，所以选择了逆序，第一个子节点放在栈尾，pop后先出来。
    let mut node_stack = Vec::new();
    node_stack.push((tree.root_node(), 0));

    let mut result_map: HashMap<usize, String> = HashMap::new();

    // while let Some((node, depth)) = node_stack.first().copied() {
    while let Some((node, depth)) = node_stack.pop() {
        // 节点类型名称
        let type_name = node.kind();
        // 节点范围（起始行、起始列、结束行、结束列）
        let range = node.range();
        for line_no in &line_numbers {
            if *line_no>=range.start_point.row+1  && *line_no<=range.end_point.row+1 
                && (type_name == "function_definition") {
                    // 提取代码片段
                    println!("we start extract code!");
                    let start_byte = node.start_byte();
                    let end_byte = node.end_byte();
                    let snippet = &source_code[start_byte..end_byte];
                    result_map.insert(node.id(), String::from(snippet));
            }
        }
        let mut children: Vec<Node> = node.children(&mut cursor).collect();
        children.reverse();
        for child in children {
            node_stack.push((child, depth + 1));
        }
    }
    let code_results: Vec<String> = result_map.into_values().collect();
    Ok(code_results)
}

pub async fn get_source_line(filepath: &str, _language: &str, line_number: usize) -> Result<String, Box<dyn Error>>{
    let source_code = fs::read_to_string(filepath).await?;

    // 按行分割源代码
    let lines: Vec<&str> = source_code.lines().collect();

    // 检查行号是否在有效范围内（1-based index）
    if line_number == 0 || line_number > lines.len() {
        return Err(format!("行号 {} 超出有效范围 (1-{})", line_number, lines.len()).into());
    }

    // 返回指定行的源代码（line_number 是 1-based，所以需要减 1）
    let line_content = lines[line_number - 1].to_string();

    Ok(line_content)
}