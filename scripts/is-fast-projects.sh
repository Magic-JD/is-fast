# A number of scripts that I created to show the flexibility of is-fast for scripting. This is not meant to be an exhaustive list of all the things that is-fast can do,
# but rather just some examples of neat functions that I put together to show how you could use this tool in your workflow.

# Check stock prices using is-fast. Args must be a stock symbol (e.g. AAPL).
# Insert the stock symbol into the url
# Select span elements with the base class
# We want this output to display directly in the terminal, rather than being shown in the tui so we use --piped.
# By default these spans are not colored, but if displaying in the terminal it is fine to include ansi-codes
isf_stock() {
    is-fast \
        --direct "https://finance.yahoo.com/quote/${1}/" \
        --selector "section.container > h1, span.base" \
        --piped \
        --no-cache \
        --pretty-print="margin:5"
}

# What is something? Give it a word or a name and it will return the first wikipedia paragraph of that thing. This will work if there is a wikipedia article with that
# exact name. Works for most people and things. E.g. isf_what albert einstein
isf_what() {
    is-fast \
        --direct "en.wikipedia.org/wiki/${*}" \
        --selector "div.mw-content-ltr > p" \
        --color=always \
        --piped \
        --nth-element 1 \
        --pretty-print="margin:20"
# We get the first paragraph with content only from the child p's of div.mw-content-ltr
# note: the first paragraph can be achieved with css selectors only, but is sometimes empty on the site - this then avoids any issues with the selected paragraph being empty.)
}

# Search stack overflow, showing only the question and answer text. Note must use --last for this, as the history output/order is not deterministic.
isf_so() {
    QUESTION=$(is-fast ${*} --site "www.stackoverflow.com" --selector "div.question .js-post-body" --color=always --pretty-print="margin:20,title:Question" --piped --flash-cache) # Find the question content.
    ANSWER=$(is-fast --last --selector "div.accepted-answer .js-post-body" --color=always --pretty-print="margin:20,title:Answer" --piped --flash-cache) # Separately find the answer content.
    cat << EOF # Format as desired

$QUESTION
$ANSWER

EOF
}

# Get a simple definition of a word. 
# NOTE capitalization is specific for ZSH - for BASH change to ${1^}
isf_define() {
    is-fast \
        --direct "www.merriam-webster.com/dictionary/${1}" \
        --selector "div.sb" \
        --nth-element 1 \
        --color=always \
        --pretty-print="margin:20,title:${(C)1}" \
        --piped
}
