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
   1. ???
3. If we encounter a _StartTag_:
   1. If is new:
      1. Find the matching _EndTag_.
      2. If it is new:
         1. If all content is new _(test case #3)_, wrap everything in an `<ins>` tag.
         2. Otherwise _(test case #4)_:
            1. Wrap all new content in `<ins>` tags.
            2. Run this algorithm on the existing content, and wrap it in `<ins class="-moved-into">` tags.
            3. Add a `-wrapper` class to the start tag.
      3. If it is deleted:
         1. ???
      4. Otherwise _(test case #1)_:
         > We expect a deleted _StartTag_ of the same name either before or after
         > the _StartTag_ we found before.
         1. Look for a **deleted** **matching** _StartTag_ before the _EndTag_.
         2. If found:
            1. If it is before the new _StartTag_ _(test case #1)_, run this
               algorithm on the content between the deleted and the new
               _StartTag_, and wrap it in an `<ins class="-moved-outof">` tag.
            2. If it is after the new _StartTag_ _(test case #2?)_, run this
               algorithm on the content between the new and the deleted
               _StartTag_, and wrap it in an `<ins class="-moved-into">` tag.
            3. Add a `-content-changed` class to the new start tag.
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

Handling invalid HTML isn't really what I was going for anyway, but it would be
nice if it doesn't produce total garbage.

I'm probably going about this the wrong way entirely anyway, since I should just
be doing actual tree diffing (and the fact we could then use DOM operation to
wrap/modify tags is a nice bonus) but I'm too stupid to figure out how any of
that works.
