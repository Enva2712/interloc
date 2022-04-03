# Interloc

Interloc aims to solve inter-service interface validation. You can think of it like a language agnostic typechecker. This is the core tool, which only handles validation over an arbitrary format representing structural type systems. This format isn't meant to be hand-written (though there are plans to clean it up to make this easier), but instead to be generated from your source code via type reflection, compiler integrations, or language features.

<video controls>
	<source src="https://raw.githubusercontent.com/Enva2712/interloc/main/demo/demo.mp4" type="video/mp4">
	<a href="https://raw.githubusercontent.com/Enva2712/interloc/main/demo/demo.mp4">Demo</a>
</video>

## TODO

* write initial adapters (graphql? openapi?)
* cleanup serialized formats
