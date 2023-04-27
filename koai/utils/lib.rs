extern crate pyo3;
use std::collections::HashMap;
use pyo3::prelude::*;


struct Token {
    text: String,
    maps: HashMap<String, i32>,
}

struct Document {
    id: String,
    maps: HashMap<String, i32>,
}


impl Token {
    fn add_neighbour(&mut self, neighbour: String) {
        if self.maps.contains_key(&neighbour) {
            *self.maps.get_mut(&neighbour).unwrap() += 1;
        } else {
            self.maps.insert(neighbour, 1);
        }
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Token {
            text: self.text.clone(),
            maps: self.maps.clone(),
        }
    }
}


impl Document {
    fn add_neighbour(&mut self, neighbour: String) {
        if self.maps.contains_key(&neighbour) {
            *self.maps.get_mut(&neighbour).unwrap() += 1;
        } else {
            self.maps.insert(neighbour, 1);
        }
    }
}

pub struct BM25 {
    index: HashMap<String, Document>,
    token_index: HashMap<String, Token>,
    k1: f32,
    b: f32,
}

impl BM25 {
    fn new() -> Self {
        BM25 {
            index: HashMap::new(),
            token_index: HashMap::new(),
            k1: 1.2,
            b: 0.75,
        }
    }

    fn _calculate(&self, tokenized_query: Vec<String>, doc: &Document, avg_doc_length:f32) -> f32 {
        let N = self.index.len() as f32;
        let mut score = 0.0;
        for token in tokenized_query {
            if doc.maps.contains_key(&token) {
                let tf = *doc.maps.get(&token).unwrap() as f32;
                let mut idf = self.token_index.get(&token).unwrap().maps.len() as f32;
                idf = (((N - idf + 0.5) / (idf + 0.5))+1.0).ln();
                score += (tf * (1.0 + self.k1) / (tf + self.k1 * ((1.-self.b) + self.b * (doc.maps.values().sum::<i32>() as f32 / avg_doc_length)))) * idf;
            }
        };
        score
    }
    fn search(&self, tokenized_query: Vec<String>, n: i32) -> Vec<(String, f32)> {
        let avg_doc_length = self.index.iter().map(|(_, doc)| doc.maps.values().sum::<i32>()).sum::<i32>() as f32 / self.index.len() as f32;
        let mut result = self.index.iter().map(|(id, doc)| {
            (id.to_string(), self._calculate(tokenized_query.clone(), doc, avg_doc_length))
        }).collect::<Vec<_>>();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result.iter().take(n as usize)
            .map(|x|x.to_owned())
            .collect::<Vec<(String, f32)>>()
    }

    fn add_document(&mut self, id:String, tokenized_doc: Vec<String>) {
        if !self.index.contains_key(&id) {
            let mut document = Document{
                id: id.to_string(),
                maps: HashMap::new(),
            };
            for token in tokenized_doc {
                document.add_neighbour(token.to_string());
                if !self.token_index.contains_key(token.as_str()){
                    let mut token_object = Token{
                        text: token.to_string(),
                        maps: HashMap::new(),
                    };
                    token_object.add_neighbour(id.to_string());
                    self.token_index.insert(token, token_object.clone());

                } else {
                    let token_object = self.token_index.get_mut(token.as_str()).unwrap();
                    token_object.add_neighbour(id.to_string());

                }
            }
            self.index.insert(id, document);
        }

    }
    fn remove_document(&mut self, id:String) {
        self.index.remove(&id);
    }
}


#[pyfunction]
fn calculate_bm25(tokenized_queries: Vec<Vec<String>>, id_candidates:Vec<(String, Vec<String>)>, n:i32)-> PyResult<Vec<Vec<(String, f32)>>>{
    let mut bm25 = BM25::new();
    for (id, tokenized_doc) in id_candidates {
        bm25.add_document(id, tokenized_doc);
    }
    let mut result = Vec::new();
    for tokenized_query in tokenized_queries {
        result.push(bm25.search(tokenized_query, n));
    }
    Ok(result)
}

#[pymodule]
fn rs_utils(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate_bm25, m)?)?;
    Ok(())
}