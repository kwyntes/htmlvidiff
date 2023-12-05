# htmlvidiff

i accidentally clicked public and now i'm too lazy to make this repo private.

i still have to figure out the logic for processing the token diff and i have no
idea if i will manage to.

### test cases

the goal is to produce these results.

| #   | old                     | new                     | vidiff                                                                                                          |
| --- | ----------------------- | ----------------------- | --------------------------------------------------------------------------------------------------------------- |
| 1   | `some <p>text</p>`      | `<p>some text</p>`      | `<p class="-content-changed"><ins class="-moved-into">some </ins>text</p>`                                      |
| 2   | `some <p>more text</p>` | `<p>some more</p> text` | `<p class="-content-changed"><ins class="-moved-into">some </ins>more</p><ins class="-moved-outof"> text</ins>` |
| 3   | `some text`             | `some <p>more</p> text` | `some <ins><p>more</p></ins> text`                                                                              |
| 4   | `some more text`        | `some <p>more</p> text` | `some <p class="-wrapper"><ins class="-moved-into">more</ins></p> text`                                         |
| 5   | `<p>some text</p>`      | `some <p>text</p>`      | `<ins class="moved-outof">some </ins><p class="-content-changed">text</p>`                                      |

### algorithm ideas

1. Go through the diff'ed tokens.
2. If we encounter a _StringSegment_:
   1. If it is new:
      1. Wrap it in an `<ins>` tag. _#TODO: Figure out how to merge contiguous
         inserts into single `<ins>` tags._
   2. Otherwise:
      > We only want to highlight new things (for now at least).
      1. Do nothing.
3. If we encounter a _StartTag_:
   1. If it is new:
      1. Find the matching _EndTag_, taking into account deleted tokens as well.
      2. If it is new:
         1. If all content is new _(test case #3)_, wrap everything in an `<ins>` tag.
         2. Otherwise _(test case #4)_:
            1. Wrap all new content in `<ins>` tags.
            2. Run this algorithm on the existing content, and wrap it in `<ins class="-moved-into">` tags.
            3. Add a `-wrapper` class to the start tag.
      3. If it is deleted:
         1. Find the matching _EndTag_.
         2. If it is new:
            1. ???
         3. If it is deleted:
            1. Run this algorithm on the content between the deleted start and end tag, and wrap it in an `<ins class="-unwrapped -moved-outof">` tag.
         4. Otherwise _(test case #5)_:
            1. Look for a (new?) matching _StartTag_ between the deleted _StartTag_ and the _EndTag_.
            2. If found:
               1. Run this algorithm on the content between the deleted _StartTag_ and the _StartTag_ we just found, and wrap it in an `<ins class="-moved-outof">` tag.
            3. Else:
               > Invalid HTML??
               1. ???
      4. Otherwise _(test case #1)_:
         > We expect a deleted _StartTag_ of the same name either before or after
         > the _StartTag_ we found before.  
         > The case of a deleted _StartTag_ before this one is now covered by 3.i.c.4.
         1. Look for a **deleted** **matching** _StartTag_ between this one and the _EndTag_.
         2. If found: 2. Run this
            algorithm on the content between the new and the deleted
            _StartTag_, and wrap it in an `<ins class="-moved-into">` tag. 3. Add a `-content-changed` class to the new start tag.
         3. Else:
            > I think this means that the old HTML was invalid? Maybe?
            1. I think we just do nothing here?
   2. If it is deleted:
      > This should be handled already by 3.i.d. _\*should\*_.
      1. Do nothing.
4. If we encounter an _EndTag_:
   1. ???

To make test case #2 work, when finding matching tags, we also take deleted tags
into account. I suppose this should always (?) work with valid HTML, but with
invalid I don't trust this at all.

[**oh god oh no**](https://html.spec.whatwg.org/multipage/syntax.html#optional-tags)

Handling invalid HTML isn't really what I was going for anyway, but it would be
nice if it doesn't produce total garbage.

I'm probably going about this the wrong way entirely anyway, since I should just
be doing actual tree diffing (and the fact we could then use DOM operation to
wrap/modify tags is a nice bonus) but I'm too stupid to figure out how any of
that works.

---

okay so [tree diffing is apparently really complicated][xmldiff] [+[archive
link][+archive]] (which at least means there's more to it than me being an
idiot), so i think that justifies me using the algorithm described above +
sanitising the inputs by running them through a proper, spec-compliant HTML5
parser and then letting that generate HTML source code again.  
time complexity? O(h no)

[xmldiff]: https://useless-factor.blogspot.com/2008/01/matching-diffing-and-merging-xml.html
[+archive]: https://web.archive.org/web/20231204163430/https://useless-factor.blogspot.com/2008/01/matching-diffing-and-merging-xml.html

---

i messed something up and now the whole thing is fucked so here's a sort of v2 (more like a v0.0.2) of the algorithm.

test case #2 will produce the following token stream:
```
+<p>
 "some"
 " "
-<p>
 "more"
+</p>
 " "
 "text"
-</p>
```

1. Go through the token diff;
2. When we encounter a _StringSegment_:
   1. #TODO
3. When we encounter a _StartTag_:
   1. Look for the matching _EndTag_, disregarding the diff tags.
   2. If not found, we're dealing with invalid HTML (right??) so either error out or silently skip this _StartTag_.
   3. Otherwise, if the _StartTag_ is new:
      1. If the _EndTag_ is new:
         1. #TODO
      2. If the _EndTag_ is deleted:
         > - The _EndTag_ could have been moved further.
         > - ...
         1. #TODO