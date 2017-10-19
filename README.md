Implementing naive Run Length Encoding (RLE) as an exercise before attempting
Consistent Overhead Byte Stuffing (COBS).

An RLE decoder is simpler in the sense that it takes a stream of bytes and
produces a stream of bytes. COBS decoders on the other hand take a stream of
bytes and produce a stream of frames/messages, these frames themselves are
streams of bytes. I suppose it is similar to a text based line splitter in that
regard.

## Streaming

Because computers have limited memory, it is sometimes desirable to process data
in chunks. When we have all the data in memory, most algorithms iterate over the
entire input. When processing the data in chunks, we have to save the
intermediate state of this loop. This makes the implementation more complex
(unfortunately) and the performance worse (as expected).

The size of these chunks and the state can be static or dynamic. The choices
affect performance and implementation and API complexity (error handling).

## Embedded

There might be embedded applications that can make use of the byte processsing
you are implementing. This means a higher valuation of performance and possibly
the absence of dynamic allocation, presence of a Floating Point Unit FPU.

Because of the wildly varying constraints, it is probably better to develop the
library for desktop first. When you have concrete examples of what you would
like to see for an embedded version can you make the right design decisions.
