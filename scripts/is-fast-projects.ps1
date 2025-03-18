# Check stock prices using is-fast. Args must be a stock symbol (e.g., AAPL).
function isf_stock {
    param (
        [string]$symbol
    )
    is-fast `
        --direct "https://finance.yahoo.com/quote/$symbol/" `
        --selector "section.container > h1, span.base" `
        --piped `
        --no-cache `
        --pretty-print="margin:5"
}

# What is something? Give it a word or a name, and it will return the first Wikipedia paragraph of that thing.
function isf_what {
    param (
        [string]$query
    )
    is-fast `
        --direct "https://en.wikipedia.org/wiki/$query" `
        --selector "div.mw-content-ltr > p" `
        --color=always `
        --piped `
        --nth-element 1 `
        --pretty-print="margin:20"
}

# Search Stack Overflow, showing only the question and answer text.
function isf_so {
    param (
        [string]$query
    )
    $QUESTION = is-fast $query --site "www.stackoverflow.com" --selector "div.question .js-post-body" --color=always --pretty-print="margin:20,title:Question" --piped --flash-cache
    $ANSWER = is-fast --last --selector "div.accepted-answer .js-post-body" --color=always --pretty-print="margin:20,title:Answer" --piped --flash-cache
    Write-Output @"

$QUESTION
$ANSWER

"@
}

# Get a simple definition of a word.
function isf_define {
    param (
        [string]$word
    )
    is-fast `
        --direct "https://www.merriam-webster.com/dictionary/$word" `
        --selector "div.sb" `
        --nth-element 1 `
        --color=always `
        --pretty-print="margin:20,title:$($word.ToUpper())" `
        --piped
}

# Check the current number of stars in the repo.
function isf_stars {
    is-fast `
        --direct "https://github.com/Magic-JD/is-fast" `
        --selector "span#repo-stars-counter-star" `
        --pretty-print="title:Current Stars,margin:5" `
        --color=always `
        --piped `
        --no-cache
}

# Checks the Google page to get the information for the info box.
function isf_quick {
    param (
        [string]$query
    )
    is-fast `
        --direct "https://www.google.com/search?q=$query" `
        --piped `
        --selector="div.ezO2md" `
        --ignore="a" `
        --no-block `
        --nth-element 1 `
        --pretty-print="margin:20" `
        --color=always `
        --no-cache
}
