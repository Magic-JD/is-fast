# Check stock prices using is-fast. Args must be a stock symbol (e.g. AAPL).
isf_stock() {
    is-fast \
        --direct "https://finance.yahoo.com/quote/${1}/" \
        --selector "span.base" \
        --piped \
        --color=always
}

# What is something? Give it a word or a name and it will return the first wikipedia paragraph of that thing.
isf_what() {
    is-fast \
        --direct "en.wikipedia.org/wiki/${*}" \
        --selector "div.mw-content-ltr > p" \
        --color=always \
        --piped \
        --element-nth 1
}

# Search stack overflow, showing only the question and answer text. Note must use --last for this, as the history output/order is not deterministic.
isf_so() {
    QUESTION=$(is-fast ${*} --site "www.stackoverflow.com" --selector "div.question .js-post-body" --color=always --piped)
    ANSWER=$(is-fast --last --selector "div.accepted-answer .js-post-body" --color=always --piped)
    cat << EOF
QUESTION:

$QUESTION

ANSWER:

$ANSWER
EOF
}

# Get a simple definition of a word.
isf_define() {
    is-fast \
        --direct "www.merriam-webster.com/dictionary/${1}" \
        --selector "div.sb" \
        --element-nth 1 \
        --color=always \
        --piped
}
