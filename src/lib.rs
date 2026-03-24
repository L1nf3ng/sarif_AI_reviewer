pub mod ai_chat;
pub mod sarif_reader;
pub mod source_reader;

pub use sarif_reader::{build_vulnerability_summary, format_for_llm, TaintStep, VulnerabilitySummary};


#[cfg(test)]
mod test_package{
    use super::source_reader;
    #[test]
    fn test_parse_python(){
        source_reader::parse_source_string();
    }
    
    #[tokio::test]
    async fn test_parse_pythonfile(){
        let input_file = "D:/PythonProjects/Archery/sql/data_dictionary.py";
        let results = source_reader::parse_source_file(input_file, "python", vec![146,148]).await.unwrap();
        for res in results{
            println!("源代码片段为：{}", res);
        }
    }
    
    #[tokio::test]
    async fn test_get_specific_line_code(){
        let input_file = "D:/PythonProjects/Archery/sql/data_dictionary.py";
        let result = source_reader::get_source_line(input_file, "python", 148).await.unwrap();
        println!("源代码行为为：{}", result);
    }
}
