use anyhow::Result;
use std::{fs::read_to_string, path::PathBuf};

pub trait File {
    fn contents(&self) -> String;
    fn from_filepath(path: PathBuf) -> Result<Self>
    where
        Self: Sized;
    fn parse(&self) -> Vec<String>;
}

pub enum FileSource {
    Filepath(std::path::PathBuf),
    S3,
}

pub struct MarkdownFile {
    pub source: FileSource,
    pub contents: String,
}

pub struct CSVFile {
    pub source: FileSource,
    pub contents: String,
}

pub struct ParagraphTextSplitter {
    pub source: FileSource,
    pub contents: String,
}

impl File for ParagraphTextSplitter {
    fn contents(&self) -> String {
        self.contents.clone()
    }

    fn from_filepath(path: PathBuf) -> anyhow::Result<Self> {
        let source = FileSource::Filepath(path.to_owned());

        let contents = read_to_string(path)?;

        Ok(Self { source, contents })
    }

    fn parse(&self) -> Vec<String> {
        self.contents
            .split("/n/n")
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
    }
}

impl File for CSVFile {
    fn contents(&self) -> String {
        self.contents.clone()
    }
    fn from_filepath(path: PathBuf) -> anyhow::Result<Self> {
        let source = FileSource::Filepath(path.to_owned());

        let contents = read_to_string(path)?;

        Ok(Self { source, contents })
    }
    fn parse(&self) -> Vec<String> {
        self.contents.lines().map(|x| x.to_owned()).collect()
    }
}

impl File for MarkdownFile {
    fn contents(&self) -> String {
        self.contents.clone()
    }
    fn from_filepath(path: PathBuf) -> anyhow::Result<Self> {
        let source = FileSource::Filepath(path.to_owned());

        let contents = read_to_string(path)?;

        Ok(Self { source, contents })
    }
    fn parse(&self) -> Vec<String> {
        let mut contents = Vec::new();
        let mut state = MarkdownFileState::None;
        let mut sentence = String::new();

        for line in self.contents.lines() {
            match state {
                MarkdownFileState::None => {
                    if line.starts_with("```") {
                        state = MarkdownFileState::CodeBlock;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    } else if line.starts_with("---") {
                        state = MarkdownFileState::Comments;
                    } else if !line.starts_with('#') && !line.is_empty() {
                        state = MarkdownFileState::Sentence;
                        sentence = String::new();
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
                MarkdownFileState::CodeBlock => {
                    sentence.push_str(line);
                    if line.starts_with("```") {
                        contents.push(sentence);
                        sentence = String::new();
                        state = MarkdownFileState::None;
                    }
                }
                MarkdownFileState::Comments => {
                    if line.starts_with("---") {
                        state = MarkdownFileState::None;
                    }
                }
                MarkdownFileState::Sentence => {
                    if line.is_empty() {
                        state = MarkdownFileState::None;
                        contents.push(sentence);
                        sentence = String::new();
                    } else {
                        sentence.push_str(line);
                        sentence.push('\n');
                    }
                }
            }
        }
        contents
    }
}

enum MarkdownFileState {
    None,
    CodeBlock,
    Sentence,
    Comments,
}
