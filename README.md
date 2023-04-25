# OpenBook V2


## License

See the LICENSE file.

The majority of this repo is MIT licensed, but some parts needed for compiling
the solana program are under GPL.

All GPL code is gated behind the `enable-gpl` feature. If you use the `openbook-v2`
crate as a dependency with the `client` or `cpi` features, you use only MIT
parts of it.

The intention is for you to be able to depend on the `openbook-v2` crate for
building closed-source tools and integrations, including other solana programs
that call into the openbook program.

But deriving a solana program with similar functionality to the openbook program
from this codebase would require the changes and improvements to stay publicly
available under GPL.

## Technical details
This program is based on [Mango V4](https://github.com/blockworks-foundation/mango-v4) and the [previous OpenBook program](https://github.com/openbook-dex/program) (which was a fork of [Serum])

The program is using anchor 0.27.0.


### Submodules

After cloning this repo you'll need to init and update its git submodules.
Consider setting the git option `submodule.recurse=true`.
