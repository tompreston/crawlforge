# Crawlforge
Crawlforge crawls git forge websites (GitHub, OpenGrok) and creates an index of
raw file URLs, which can then be downloaded using wget. This is useful when you
have access to the forge website, but not any sort of archive or git CLI access.

You should really try to get access to source code archives or git CLI before
resorting to using this tool.

Don't bother implementing paralleisation with threads because we can just start
a new crawlforge process for each subdirectory.

    cargo run -- https://github.com/tompreston/sup/ | tee index.txt
    cargo run -- -f opengrok http://username:password@opengrok.com/whatever/ | tee index.txt

    # if necessary, combine several indexes which have been created in parallel
    sort index1.txt index2.txt > index.txt

    # Serial wget
    wget \
        --http-user=foo \
        --http-password=bar \
        --force-directories \
        --continue \
        --input-file=index.txt

    # Parallel wget (untested)
    cat index.txt | parallel --gnu wget \
        --http-user=foo \
        --http-password=bar \
        --force-directories \
        --continue
