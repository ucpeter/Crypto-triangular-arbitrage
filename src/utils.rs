use std::collections::HashMap;

pub fn build_graph(markets: &Vec<(String, String)>) -> HashMap<String, Vec<String>> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for (base, quote) in markets {
        graph.entry(base.clone()).or_default().push(quote.clone());
        graph.entry(quote.clone()).or_default().push(base.clone());
    }
    graph
}
