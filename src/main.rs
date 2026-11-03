use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::HashMap;
use std::path::PathBuf;

// ===================== Terminal Values =====================
pub const HAI: &str = "#HAI";
pub const KBYE: &str = "#KBYE";
pub const OBTW: &str = "#OBTW";
pub const TLDR: &str = "#TLDR";
pub const MAEK: &str = "#MAEK";
pub const OIC: &str = "#OIC";
pub const GIMMEH: &str = "#GIMMEH";
pub const MKAY: &str = "#MKAY";
pub const HEAD: &str = "HEAD";
pub const TITLE: &str = "TITLE";
pub const PARAGRAF: &str = "PARAGRAF";
pub const BOLD: &str = "BOLD";
pub const LIST: &str = "LIST";
pub const ITEM: &str = "ITEM";
pub const NEWLINE: &str = "#NEWLINE";
pub const LINX: &str = "LINX";
pub const IHAZ: &str = "#IHAZ";
pub const ITIZ: &str = "#ITIZ";
pub const VAR: &str = "VAR";
 
/// Every fixed terminal, handy for lexer lookups.
pub const ALL_FIXED: [&str; 19] = [
    HAI, KBYE, OBTW, TLDR, MAEK, OIC, GIMMEH, MKAY, HEAD, TITLE, PARAGRAF,
    BOLD, LIST, ITEM, NEWLINE, LINX, IHAZ, ITIZ, VAR,
];
 
/// True if `token` exactly matches one of the fixed terminals.
pub fn is_fixed_terminal(token: &str) -> bool {
    ALL_FIXED.contains(&token)
}
 
// ---------- Pattern terminals (functions) ----------
 
/// VARNAME ::= any single word (A-Z, a-z, no spaces)
pub fn is_varname(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic())
}
 
/// Characters allowed in a VARVALUE. Adjust this list to match your spec.
fn is_varvalue_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(c, ' ' | ',' | '.' | '"' | '\'' | ':' | '?' | '!' | '%' | '/' | '-' | '_')
}
 
/// VARVALUE ::= allowed text characters
pub fn is_varvalue(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_varvalue_char)
}
 
/// TEXT ::= plain text (letters, digits, punctuation, spaces, tabs, newlines)
pub fn is_text(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            !matches!(c, '#')
                && (c.is_ascii_alphanumeric()
                    || c.is_ascii_punctuation()
                    || matches!(c, ' ' | '\t' | '\n' | '\r'))
        })
}
 
/// ADDRESS ::= text without spaces
pub fn is_address(s: &str) -> bool {
    is_text(s) && !s.chars().any(|c| c.is_whitespace())
}


//
// ===================== Compiler Trait =====================
//

/// Trait for a simple lolcompiler front-end. 
/// Errors should cause immediate exit inside the implementation.
pub trait Compiler {
    /// Begin the compilation process (entry point).
    fn compile(&mut self, source: &str);

    /// Get the next token from the lexical analyzer.
    fn next_token(&mut self) -> String;

    /// Run the syntax analyzer starting from <lolcode>.
    fn parse(&mut self);

    /// Get the current token being processed.
    fn current_token(&self) -> String;

    /// Set the current token (typically used internally).
    fn set_current_token(&mut self, tok: String);

    // Get the next token to be processed.
    fn peek_token(&self) -> String;
}

//
// ===================== Lexical Analyzer =====================
//

/// Trait for a simple lexical analyzer.
/// Implements a character-by-character analysis
/// from a state machine design.
pub trait LexicalAnalyzer {
    /// Return the next character from the input.
    /// If input is exhausted, should terminate the program.
    fn get_char(&mut self) -> char;

    /// Add a character to the current potential token.
    fn add_char(&mut self, c: char);

    /// Lookup a potential token to determine if it is valid.
    /// Returns true if a valid token/lexeme, false otherwise.
    fn lookup(&self, s: &str) -> bool;
}

/// A concrete implementation of the lexical analyzer.
pub struct SimpleLexicalAnalyzer {
    input: Vec<char>,
    position: usize,
    current_build: String,
    pub tokens: Vec<String>,
    pub positions: Vec<(usize, usize)>,
    line: usize,
    col: usize,
    start: (usize, usize),
}

impl SimpleLexicalAnalyzer {
    pub fn new(source: &str) -> Self {
        Self {
            input: source.chars().collect(),
            position: 0,
            current_build: String::new(),
            tokens: Vec::new(),
            positions: Vec::new(),
            line: 1,
            col: 1,
            start: (1, 1),
        }
    }

    //checking current build and pushing to a token to tokens was done twice; this reduces redundancy
    //also the starting position of a token to the vector positions
    fn flush(&mut self) {
        if !self.current_build.is_empty() {
            self.tokens.push(std::mem::take(&mut self.current_build));
            self.positions.push(self.start);
        }
    }

    pub fn tokenize(&mut self) {
        loop {
            let c = self.get_char();
            if c == '\0' {
                break;
            }
            if c.is_whitespace() {
                self.flush();
                //moves position in conjuction with each character collected from get_char()
                if c == '\n' { self.line += 1; self.col = 1; } else { self.col += 1; }
            } else {
                //indicates start of new token and the start of a new token position; position is tracked by self.start
                if self.current_build.is_empty() {
                    self.start = (self.line, self.col);
                }
                self.add_char(c);
                self.col += 1;
            }
        }

        self.flush();
        self.tokens.reverse();
        self.positions.reverse(); //reverses the corresponding position of each token to match the reversing of the tokens
    }
}

impl LexicalAnalyzer for SimpleLexicalAnalyzer {
    fn get_char(&mut self) -> char {
        if self.position < self.input.len() {
            let c = self.input[self.position];
            self.position += 1;
            c
        } else {
            '\0'
        }
    }

    fn add_char(&mut self, c: char) {
        self.current_build.push(c);
    }

    fn lookup(&self, s: &str) -> bool {
        is_fixed_terminal(s) ||
        is_varname(s) ||
        is_varvalue(s) ||
        is_text(s) ||
        is_address(s)
    }
    
}

//
// ===================== Syntax Analyzer =====================
//


/// OPTION 2 - Trait for a recursive descent Syntax Analyzer 
/// over Vec<String>. Each function parses a nonterminal in 
/// the grammar. On error: return Err(message), on success: Ok(()).
pub trait SyntaxAnalyzer {
    fn new() -> Self;
    fn start(&mut self);
    fn parse_lolcode(&mut self) -> Result<(), String>;        
    fn parse_head(&mut self) -> Result<(), String>;           
    fn parse_title(&mut self) -> Result<(), String>;          
    fn parse_comment(&mut self) -> Result<(), String>;        
    fn parse_body(&mut self) -> Result<(), String>; 
    fn parse_inner_body(&mut self) -> Result<(), String>;     
    fn parse_paragraph(&mut self) -> Result<(), String>;      
    fn parse_inner_paragraph(&mut self) -> Result<(), String>;
    fn parse_inner_text(&mut self) -> Result<(), String>;     
    fn parse_variable_define(&mut self) -> Result<(), String>;
    fn parse_variable_use(&mut self) -> Result<(), String>;   
    fn parse_bold(&mut self) -> Result<(), String>;           
    fn parse_list(&mut self) -> Result<(), String>;           
    fn parse_list_items(&mut self) -> Result<(), String>;     
    fn parse_list_item(&mut self) -> Result<(), String>;
    fn parse_inner_list(&mut self) -> Result<(), String>;     
    fn parse_list_content(&mut self) -> Result<(), String>;
    fn parse_link(&mut self) -> Result<(), String>;          
    fn parse_newline(&mut self) -> Result<(), String>;        
    fn parse_text(&mut self) -> Result<(), String>;           
}


/// <sentence>    ::= <noun_phrase> [<adverb>] <verb> <noun_phrase>
/// <noun_phrase> ::= <article> <adjective> <noun>
pub struct LolCodeCompiler {
    lexer: SimpleLexicalAnalyzer,
    current_tok: String,
    current_pos: (usize, usize), //current position of token by line and column being examine by parser
    scopes: Vec<Scope>, //vector of scopes; new scopes are pushed onto vector; when a scope ends it is popped
    closed: HashMap<String, VarInfo>, //variables whose scope already ended (for nicer errors)
    source_lines: Vec<String>, //lines of code from the lol file used for displaying errors
    filename: String, //file name for more detailed error messages
    parse_tree: String //parse tree translated to HTML
}

impl SyntaxAnalyzer for LolCodeCompiler {

    fn new() -> Self {
        Self {
            lexer: SimpleLexicalAnalyzer::new(""),
            current_tok: String::new(),
            current_pos: (1, 1), //explain this and below
            scopes: Vec::new(),
            closed: HashMap::new(),
            source_lines: Vec::new(),
            filename: String::new(),
            parse_tree: String::new(),
        }
    }

    fn start(&mut self) {
        let candidate = self.lexer.tokens.pop().unwrap_or_default();
        let pos = self.lexer.positions.pop().unwrap_or((1, 1)); //consume a position from the positions vector for each token consumed from the tokens vector
        if self.lexer.lookup(&candidate.to_uppercase()) { //changed cadidate to uppercase so that compiler is case insensitive
            self.current_tok = candidate;
            self.current_pos = pos;
        } else if !candidate.is_empty() {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate);
            std::process::exit(1);
        } else {
            eprintln!("User error: The provided sentence is empty.");
            std::process::exit(1);
        }
    }

    //<lolcode> ::= HAI <comments> <head> <optional-variable-define> <body> KBYE
    fn parse_lolcode(&mut self) -> Result<(), String> {

        //HAI
        if &self.current_tok.to_uppercase() == HAI {
            let _ = self.next_token();
            self.push_scope();
            self.parse_tree.push_str("<html>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #HAI was expected",
                self.current_tok
            ));
        }

        //<comments>
        while &self.current_tok.to_uppercase() == OBTW {
            if &self.current_tok.to_uppercase() == OBTW {
                self.parse_comment()?;
            } else {
                break;
            }
        }

        //<head>
        if &self.current_tok.to_uppercase() == MAEK && &self.peek_token().to_uppercase() == HEAD{
            self.parse_head()?;
        }

        //<optional-variable-define>
        if &self.current_tok.to_uppercase() == IHAZ {
            self.parse_variable_define()?;
        }
        
        //<body>
        self.parse_tree.push_str("\n<body>");
         self.parse_body()?;
         self.parse_tree.push_str("\n</body>");

        //KBYE
        if &self.current_tok.to_uppercase() == KBYE {
            let _ = self.next_token();
            self.pop_scope();
            self.parse_tree.push_str("\n</html>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #KBYE was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<head> ::= MAEK HEAD <title> MKAY | ε
    fn parse_head(&mut self) -> Result<(), String> {
        //MAEK
        if &self.current_tok.to_uppercase() == MAEK {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MAEK was expected",
                self.current_tok
            ));
        }

        //HEAD
        if &self.current_tok.to_uppercase() == HEAD {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<head>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when HEAD was expected",
                self.current_tok
            ));
        }

        //<title>
        if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == TITLE{
            self.parse_title()?;
        }

        //MKAY
        if &self.current_tok.to_uppercase() == MKAY {
            let _ = self.next_token();
            self.parse_tree.push_str("\n</head>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MKAY was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<title> ::= GIMMEH TITLE TEXT OIC
    fn parse_title(&mut self) -> Result<(), String> {
        //GIMMEH
        if &self.current_tok.to_uppercase() == GIMMEH {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #GIMMEH was expected",
                self.current_tok
            ));
        }

        //TITLE
        if &self.current_tok.to_uppercase() == TITLE {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<title>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when TITLE was expected",
                self.current_tok
            ));
        }

        //TEXT
        if is_text(&self.current_tok) {
            self.parse_text()?;
        }

        //OIC
        if &self.current_tok.to_uppercase() == OIC {
            let _ = self.next_token();
            self.parse_tree.push_str("</title>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when OIC was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<comment> ::= OBTW TEXT TLDR
    fn parse_comment(&mut self) -> Result<(), String> {
        //OBTW
        if &self.current_tok.to_uppercase() == OBTW {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<!--");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #OBTW was expected",
                self.current_tok
            ));
        }

        //TEXT
        if is_text(&self.current_tok) {
            self.parse_text()?;
        }

        //TLDR
        if &self.current_tok.to_uppercase() == TLDR {
            let _ = self.next_token();
            self.parse_tree.push_str("-->");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #TLDR was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<body> ::= <inner-body> <body> | ε
    fn parse_body(&mut self) -> Result<(), String> {
        while &self.current_tok.to_uppercase() != KBYE {
            let previous_token = self.current_tok.clone();
            let current_position = self.lexer.tokens.len();

            self.parse_inner_body()?;

            if self.current_tok == previous_token &&
               current_position == self.lexer.tokens.len() {
                return Err(format!(
                    "Syntax error: {} was found unexpectedly",
                    self.current_tok
                ));
            }
        }

        Ok(())
    }

    //<inner-body> ::= <paragraph> | <comment> | <bold> | <list> | <link> | <newline> | <variable-use> | TEXT
    fn parse_inner_body(&mut self) -> Result<(), String> {
        //<paragraph>
        if &self.current_tok.to_uppercase() == MAEK && &self.peek_token().to_uppercase() == PARAGRAF {
            self.parse_paragraph()?;
        }

        //<inner-text> ::= <variable-use> | <comment> | <bold> | <list> | <link> | <newline> | TEXT
        else {
            self.parse_inner_text()?;
        }

        Ok(())
    }

    //<paragraph> ::= MAEK PARAGRAF <optional-variable-define> <inner-paragraph> MKAY
    fn parse_paragraph(&mut self) -> Result<(), String> {
        //MAEK
        if &self.current_tok.to_uppercase() == MAEK {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MAEK was expected",
                self.current_tok
            ));
        }

        //PARAGRAF
        if &self.current_tok.to_uppercase() == PARAGRAF {
            let _ = self.next_token();
            self.push_scope();
            self.parse_tree.push_str("\n<p>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when PARAGRAF was expected",
                self.current_tok
            ));
        }

        //<optional-variable-define>
        if &self.current_tok.to_uppercase() == IHAZ {
            self.parse_variable_define()?;
        }

        //<inner-paragraph>
        self.parse_inner_paragraph()?;

        //MKAY
        if &self.current_tok.to_uppercase() == MKAY {
            let _ = self.next_token();
            self.parse_tree.push_str("\n</p>");
            self.pop_scope();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MKAY was expected",
                self.current_tok
            ));
        }

        Ok(())
    }
    
    //<inner-paragraph> ::= <inner-text> <inner-paragraph> | ε
    fn parse_inner_paragraph(&mut self) -> Result<(), String> {
        //<inner-text>
        while &self.current_tok.to_uppercase() != MKAY {
            let previous_token = self.current_tok.clone();
            let current_position = self.lexer.tokens.len();

            self.parse_inner_text()?;

            if self.current_tok == previous_token &&
               current_position == self.lexer.tokens.len() {
                return Err(format!(
                    "Syntax error: {} was found unexpectedly",
                    self.current_tok
                ));
            }
        }

        Ok(())
    }

    //<inner-text> ::= <variable-use> | <comment> | <bold> | <list> | <link> | <newline> | TEXT
    fn parse_inner_text(&mut self) -> Result<(), String> {
        //<variable-use>
        if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == VAR {
            self.parse_variable_use()?;
        }

        //<comment>
        else if &self.current_tok.to_uppercase() == OBTW {
            self.parse_comment()?;
        }

        //<bold>
        else if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == BOLD {
            self.parse_bold()?;
        }

        //<list>
        else if &self.current_tok.to_uppercase() == MAEK && &self.peek_token().to_uppercase() == LIST {
            self.parse_list()?;
        }

        //<link>
        else if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == LINX {
            self.parse_link()?;
        }

        //<newline>
        else if &self.current_tok.to_uppercase() == NEWLINE {
            self.parse_newline()?;
        }

        //TEXT
        else if is_text(&self.current_tok) {
            self.parse_text()?;
        }

        //Unrecognized/Out-of-order Token
        else {
            return Err(format!(
                "Syntax error: {} was found unexpectedly",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<variable-define> ::= IHAZ VARNAME ITIZ VARVALUE MKAY
    fn parse_variable_define(&mut self) -> Result<(), String> {
        //IHAZ
        if &self.current_tok.to_uppercase() == IHAZ {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #IHAZ was expected",
                self.current_tok
            ));
        }

        //VARNAME
        let var_name = self.current_tok.clone();
        let var_pos = self.current_pos;
        if is_varname(&self.current_tok) {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when variable name was expected",
                self.current_tok
            ));
        }

        //ITIZ
        if &self.current_tok.to_uppercase() == ITIZ {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #ITIZ was expected",
                self.current_tok
            ));
        }

        //VARVALUE
        let mut var_value: String = "".to_string();
        if is_varvalue(&self.current_tok) {
            while is_varvalue(&self.current_tok) && &self.current_tok.to_uppercase() != MKAY { //process multi-word variable contents
                let temp_value = &self.current_tok;
                var_value.push_str(" ");
                var_value.push_str(&temp_value);
                let _ = self.next_token();
            }
            var_value.push_str(" ");
        } else {
            return Err(format!(
                "Syntax error: {} was found when variable value was expected",
                self.current_tok
            ));
        }

        //MKAY
        if &self.current_tok.to_uppercase() == MKAY {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MKAY was expected",
                self.current_tok
            ));
        }

        // Only visible AFTER the definition is complete
        self.declare_var(var_name, var_value, var_pos);

        Ok(())
    }

    //<variable-use> ::= GIMMEH VAR VARNAME OIC
    fn parse_variable_use(&mut self) -> Result<(), String> {
        //GIMMEH
        if &self.current_tok.to_uppercase() == GIMMEH {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #GIMMEH was expected",
                self.current_tok
            ));
        }

        //VAR
        if &self.current_tok.to_uppercase() == VAR {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when VAR was expected",
                self.current_tok
            ));
        }

        //VARNAME
        if is_varname(&self.current_tok) {
            let name = self.current_tok.clone();
            if self.lookup_var(&name).is_none() {
                return Err(self.undefined_var_error(&name, self.current_pos));
            }
            let var_value = self.get_var_value(&name).unwrap();
            self.parse_tree.push_str(&var_value);
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when variable name was expected",
                self.current_tok
            ));
        }

        //OIC
        if &self.current_tok.to_uppercase() == OIC {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #OIC was expected",
                self.current_tok
            ));
        }

        Ok(())
    }
    
    //<bold> ::= GIMMEH BOLD TEXT OIC
    fn parse_bold(&mut self) -> Result<(), String> {
        //GIMMEH
        if &self.current_tok.to_uppercase() == GIMMEH {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #GIMMEH was expected",
                self.current_tok
            ));
        }

        //BOLD
        if &self.current_tok.to_uppercase() == BOLD {
            let _ = self.next_token();
            self.parse_tree.push_str("<b>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when BOLD was expected",
                self.current_tok
            ));
        }

        //TEXT
        if is_text(&self.current_tok) {
            self.parse_text()?;
        }

        //OIC
        if &self.current_tok.to_uppercase() == OIC {
            let _ = self.next_token();
            self.parse_tree.push_str("</b>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #OIC was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<list> ::= MAEK LIST <list-item> <list-items> MKAY
    fn parse_list(&mut self) -> Result<(), String> {
        //MAEK
        if &self.current_tok.to_uppercase() == MAEK {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MAEK was expected",
                self.current_tok
            ));
        }

        //LIST
        if &self.current_tok.to_uppercase() == LIST {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<ul>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when LIST was expected",
                self.current_tok
            ));
        }

        //<list-item>
        self.parse_list_item()?;

        //<list-items>
        self.parse_list_items()?;

        //MKAY
        if &self.current_tok.to_uppercase() == MKAY {
            let _ = self.next_token();
            self.parse_tree.push_str("\n</ul>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #MKAY was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<list-item> ::= GIMMEH ITEM <optional-variable-define> <inner-list> OIC
    fn parse_list_item(&mut self) -> Result<(), String> {
        //GIMMEH
        if &self.current_tok.to_uppercase() == GIMMEH {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #GIMMEH was expected",
                self.current_tok
            ));
        }

        //ITEM
        if &self.current_tok.to_uppercase() == ITEM {
            let _ = self.next_token();
            self.push_scope();
            self.parse_tree.push_str("\n<li>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when ITEM was expected",
                self.current_tok
            ));
        }

        //<optional-variable-define>
        if &self.current_tok.to_uppercase() == IHAZ {
            self.parse_variable_define()?;
        }

        //<inner-list>
        self.parse_inner_list()?;

        //OIC
        if &self.current_tok.to_uppercase() == OIC {
            let _ = self.next_token();
            self.pop_scope();
            self.parse_tree.push_str("</li>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #OIC was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<<list-items> ::= <list-item> <list-items> | ε
    fn parse_list_items(&mut self) -> Result<(), String> {
        while &self.current_tok.to_uppercase() != MKAY {
            let previous_token = self.current_tok.clone();
            let current_position = self.lexer.tokens.len();

            self.parse_list_item()?;

            if self.current_tok == previous_token && 
               current_position == self.lexer.tokens.len() {
                return Err(format!(
                    "Syntax error: {} was found unexpectedly",
                    self.current_tok
                ));
            }
        }

        Ok(())
    }

    //<inner-list> ::= <list-content> <inner-list> | ε
    fn parse_inner_list(&mut self) -> Result<(), String> {
        while &self.current_tok.to_uppercase() != OIC {
            let previous_token = self.current_tok.clone();
            let current_position = self.lexer.tokens.len();

            self.parse_list_content()?;

            if self.current_tok == previous_token &&
               current_position == self.lexer.tokens.len() {
                return Err(format!(
                    "Syntax error: {} was found unexpectedly",
                    self.current_tok
                ));
            }
        }
        Ok(())
    }

    //<list-content> ::= <bold> | <comment> | <link> | <newline> | <variable-use> | TEXT
    fn parse_list_content(&mut self) -> Result<(), String> {
        //<variable-use>
        if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == VAR {
            self.parse_variable_use()?;
        }

        //<comment>
        else if &self.current_tok.to_uppercase() == OBTW {
            self.parse_comment()?;
        }

        //<bold>
        else if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == BOLD {
            self.parse_bold()?;
        }

        //<link>
        else if &self.current_tok.to_uppercase() == GIMMEH && &self.peek_token().to_uppercase() == LINX {
            self.parse_link()?;
        }

        //<newline>
        else if &self.current_tok.to_uppercase() == NEWLINE {
            self.parse_newline()?;
        }

        //TEXT
        else if is_text(&self.current_tok) {
            self.parse_text()?;
        }

        //Unrecognized/Out-of-order Token
        else {
            return Err(format!(
                "Syntax error: {} was found unexpectedly",
                self.current_tok
            ));
        }

        Ok(())
    }

    //<link> ::= GIMMEH LINX ADDRESS OIC
    fn parse_link(&mut self) -> Result<(), String> {
        //GIMMEH
        if &self.current_tok.to_uppercase() == GIMMEH {
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when #GIMMEH was expected",
                self.current_tok
            ));
        }

        //LINX
        if &self.current_tok.to_uppercase() == LINX {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<a href=\"");
        } else {
            return Err(format!(
                "Syntax error: {} was found when LINX was expected",
                self.current_tok
            ));
        }

        //ADDRESS
        if is_address(&self.current_tok) {
            self.parse_tree.push_str(&self.current_tok);
            self.parse_tree.push_str("\">");
            self.parse_tree.push_str(&self.current_tok);
            let _ = self.next_token();
        } else {
            return Err(format!(
                "Syntax error: {} was found when address was expected",
                self.current_tok
            ));
        }

        //OIC
        if &self.current_tok.to_uppercase() == OIC {
            let _ = self.next_token();
            self.parse_tree.push_str("</a>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #OIC was expected",
                self.current_tok
            ));
        }        

        Ok(())
    }

    //<newline> ::= NEWLINE
    fn parse_newline(&mut self) -> Result<(), String> {
        //NEWLINE
        if &self.current_tok.to_uppercase() == NEWLINE {
            let _ = self.next_token();
            self.parse_tree.push_str("\n<br>");
        } else {
            return Err(format!(
                "Syntax error: {} was found when #NEWLINE was expected",
                self.current_tok
            ));
        }

        Ok(())
    }

    fn parse_text(&mut self) -> Result<(), String> {
        let mut has_some_text;
        let mut full_text: String = " ".to_string();

        if is_text(&self.current_tok) {
            full_text.push_str(&self.current_tok);
            let _ = self.next_token();
            has_some_text = true;
        } else {
            return Err(format!(
                "Syntax error: {} was found when TEXT was expected",
                self.current_tok
            ));
        }

        while has_some_text {
            if is_text(&self.current_tok) {
                full_text.push_str(" ");
                full_text.push_str(&self.current_tok);
                let _ = self.next_token();
            } else {
                has_some_text = false;
            }
        }

        full_text.push_str(" ");

        self.parse_tree.push_str(&full_text);
        Ok(())
    }
    
}

impl Compiler for LolCodeCompiler {
    fn compile(&mut self, source: &str) {
        //iterates over the lines of source code, converts them from &str to String, and adds them to the source_lines vector
        self.source_lines = source.lines().map(String::from).collect();
        self.lexer = SimpleLexicalAnalyzer::new(source);
        self.lexer.tokenize();
        self.start();
    }

    fn next_token(&mut self) -> String {
        let candidate = self.lexer.tokens.pop().unwrap_or_default();
        let pos = self.lexer.positions.pop().unwrap_or(self.current_pos); //same purpose as in start(); consume a position each token consumed
        if self.lexer.lookup(&candidate.to_uppercase()) { //changed cadidate to uppercase so that compiler is case insensitive
            self.current_tok = candidate.clone();
            self.current_pos = pos; //explain this
            candidate
        } else if self.lexer.tokens.is_empty() {
            self.current_tok.clear();
            String::new()
        } else {
            eprintln!("Lexical error: '{}' is not a recognized token.", candidate);
            std::process::exit(1);
        }
    }

    fn parse(&mut self) {
        if let Err(error) = self.parse_lolcode() {
            eprintln!("{}", error);
            std::process::exit(1);
        }
        
        if !self.lexer.tokens.is_empty() || !self.current_tok.is_empty() {
            eprintln!("Syntax error: Additional tokens found after the sentence.");
            std::process::exit(1);
        }
    }

    fn current_token(&self) -> String { self.current_tok.clone() }
    fn set_current_token(&mut self, tok: String) { self.current_tok = tok; }

    //acts as one token lookahead
    fn peek_token(&self) -> String {
        self.lexer.tokens.last().cloned().unwrap_or_default()
    }
}

//
// ===================== Semantic Analyzer =====================
//

#[derive(Clone)]
struct VarInfo { //used to store the value of a variable as well as its location by line and column
    value: String,
    line: usize,
    col: usize,
}
type Scope = HashMap<String, VarInfo>; //an alias for a hashmap representing the different scopes

impl LolCodeCompiler {
    pub fn set_filename(&mut self, f: &str) { self.filename = f.to_string(); }

    //creates a new scope and pushes it to the Scope Aliased vector
    fn push_scope(&mut self) { self.scopes.push(Scope::new()); }

    //removes most recent scope from the Scope Aliased vector
    fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for (name, info) in scope {
                self.closed.insert(name, info);
            }
        }
    }

    //returns variable from most recent scope
    fn get_var_value(&self, name: &str) -> Option<String> {
        self.lookup_var(name).map(|info| info.value.clone())
    }

    //looks up variable if it exists in any scope
    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }

    //adds variable value and position to a VarInfo object which, along with the variable name, is inserted as a key value pair into the Scopes Aliased vector
    fn declare_var(&mut self, name: String, value: String, pos: (usize, usize)) {
        let info = VarInfo { value, line: pos.0, col: pos.1 };
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, info);
        }
    }

    // Renders a rustc-style diagnostic for scoping errors
    fn diagnostic(&self, msg: &str, pos: (usize, usize), len: usize,
                  label: &str, notes: &[String]) -> String {
        let (line, col) = pos;
        let width = line.to_string().len();
        let pad = " ".repeat(width);
        let src = self.source_lines.get(line - 1).map(|s| s.as_str()).unwrap_or("");
        let mut out = format!(
            "error: {}\n{}--> {}:{}:{}\n{} |\n{} | {}\n{} | {}{} {}\n{} |\n",
            msg, pad, self.filename, line, col, pad,
            line, src,
            pad, " ".repeat(col - 1), "^".repeat(len.max(1)), label,
            pad
        );
        for n in notes {
            out.push_str(&format!("{} = {}\n", pad, n));
        }
        out
    }

    //checks for undefined variables; provides a message if they were once in scope, but scope is gone
    //otherwise, renders a diagnostic message by calling diagnostic()
    fn undefined_var_error(&self, name: &str, pos: (usize, usize)) -> String {
        let mut notes = Vec::new();
        if let Some(old) = self.closed.get(name) {
            notes.push(format!(
                "note: `{}` was defined at {}:{} but that scope has already ended",
                name, old.line, old.col
            ));
        }
        notes.push(format!(
            "help: define it first with `#IHAZ {} #ITIZ <value> #MKAY`", name
        ));
        self.diagnostic(
            &format!("use of undefined variable `{}`", name),
            pos, name.len(), "not found in this scope", &notes,
        )
    }
}



//
// ===================== Open File in Browser =====================
//

fn open_in_browser(path_or_url: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        // Windows requires passing "/c" to cmd.exe followed by the "start" command
        Command::new("cmd")
            .args(["/C", "start", "", path_or_url])
            .status()?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS natively provides the "open" command
        Command::new("open")
            .arg(path_or_url)
            .status()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux distributions use "xdg-open" to respect default desktop application settings
        Command::new("xdg-open")
            .arg(path_or_url)
            .status()?;
    }

    Ok(())
}

//
// ===================== Main =====================
//

fn main() {
    // Usage in VS Code terminal:
    // cargo run -- ./test/Test2.lol or main.exe ../test/Test2.lol

    // Code if no file is specified in the arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    //Code if there are too many arguments
    if args.len() > 2 {
        eprint!("Unknown arguments: ");
        for i in 2..args.len() {
            eprint!("{} ", args[i])
        }
        eprintln!();
        std::process::exit(1);
    }

    //Code if the file in the arguments is not found in the project root
    let filename = &args[1];
    let sentence = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    //Code if the file type is invalid
    let file = Path::new(&filename);
    let ext = file.extension().and_then(|s| s.to_str()).unwrap_or("Error reading file type");
    if ext != "lol" {
        eprintln!("Error: expected file type lol but received file type {}", ext);
        std::process::exit(1);
    }

    let mut compiler = LolCodeCompiler::new();
    compiler.set_filename(filename);
    compiler.compile(&sentence);
    compiler.parse();

    //create html file
    let html_file = PathBuf::from(file.with_extension("html")).to_string_lossy().into_owned();
    let html_content = String::from(compiler.parse_tree);
    let html_file_to_read = html_file.clone();
    fs::write( html_file, html_content).expect("Failed to write HTML file");

    //open generated HTML file in browser - from Project1 directions
    if let Err(e) = open_in_browser(&html_file_to_read) {
        eprintln!("Failed to open browser: {}", e);
    }
}
