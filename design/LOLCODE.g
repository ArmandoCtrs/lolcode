grammar LOLCODE;


//lexical definitions
LOLCODE_BEGIN		: '#HAI' | '#hai';
LOLCODE_END		: '#KBYE' | '#kbye';
HEAD_BEGIN		: '#MAEK HEAD' | '#maek head';
TITLE_BEGIN		: '#GIMMEH TITLE' | '#gimmeh title';
COMMENT_BEGIN		: '#OBTW' | '#obtw';
COMMENT_END		: '#TLDR' | '#tldr';
BOLD_BEGIN		: '#GIMMEH BOLD' | '#gimmeh bold';
PARAGRAPH_BEGIN		: '#MAEK PARAGRAF' | '#maek paragraf';
LIST_BEGIN		: '#MAEK LIST' | '#maek list';
LIST_ITEM_BEGIN		: '#GIMMEH ITEM' | '#gimmeh item';
LINK_BEGIN		: '#GIMMEH LINX' | '#gimmeh linx';
NEWLINE			: '#NEWLINE';
VARIABLE_BEGIN		: '#IHAZ' | '#ihaz';
VARIABLE_VALUE		: '#ITIZ' | '#itiz';
VARIABLE_NAME_BEGIN	: '#GIMMEH VAR' | '#gimmeh var';
OIC			: '#OIC' | '#oic';
MKAY			: '#MKAY' | '#mkay';


LETTER			: 'A'..'Z' | 'a'..'z';
NUMBER			: '0'..'9';
PUNCTUATION		: ',' | '.' | '"' | ':' | '?' | '!' | '%' | '=' | '/';
WHITESPACE		: '\t' | ' ' | '\n' | '\r';

//syntax definitions
lolcode			: LOLCODE_BEGIN WHITESPACE* comment_optional* (head WHITESPACE*)? (element WHITESPACE* comment_optional* | text WHITESPACE*)* LOLCODE_END;
head			: HEAD_BEGIN WHITESPACE* (title WHITESPACE*)? MKAY;
title			: TITLE_BEGIN WHITESPACE* (text WHITESPACE*)* OIC;

element			: paragraph | paragraph_items;
paragraph_items		: bold | list | link | newline | variable | variable_name;
comment_optional	: comment WHITESPACE*;
comment			: COMMENT_BEGIN WHITESPACE* (text WHITESPACE*)* COMMENT_END;
paragraph		: PARAGRAPH_BEGIN WHITESPACE* ((text | paragraph_items) WHITESPACE*)* MKAY;
list			: LIST_BEGIN WHITESPACE* (list_item WHITESPACE*)+ MKAY;
list_item		: LIST_ITEM_BEGIN WHITESPACE* ((text | bold | variable_name) WHITESPACE*)* OIC;
link			: LINK_BEGIN WHITESPACE* (word WHITESPACE*)* OIC;
newline			: NEWLINE;
bold			: BOLD_BEGIN WHITESPACE* ((text | variable_name) WHITESPACE*)* OIC;
text 			: word (WHITESPACE+ word)*;
word			: (LETTER | NUMBER | PUNCTUATION)+;

variable		: VARIABLE_BEGIN WHITESPACE* (LETTER | NUMBER)+ WHITESPACE* VARIABLE_VALUE WHITESPACE* text+ WHITESPACE* MKAY;
variable_name		: VARIABLE_NAME_BEGIN WHITESPACE* (LETTER | NUMBER)+ WHITESPACE* OIC;