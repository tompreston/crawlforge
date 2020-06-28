# Crawlforge
Crawlforge crawls git forge websites (GitHub, OpenGrok) and creates an index of
raw file URLs, which can then be downloaded using wget. This is useful when you
have access to the forge website, but not any sort of archive or git CLI access.

You should really try to get access to source code archives or git CLI before
resorting to using this tool.

Tasks:
- [ ] Basic parsing of single repo directory
- [ ] Parse entire repo directory

Don't bother with paralleisation because we can just start a new crawlforge
process for each subdirectory.

Psuedocode:

    parse_forge_dir(url):
        body = reqwests.get(url)
        dirs = parse_dirs(forge, body)
        files = parse_files(forge, body)

        for f in listing.files:
            print(f)

        for d in listing.dirs:
            parse_forge_dir(d)

    main():
        parse_forge_dir(url)
