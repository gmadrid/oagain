Given the complexity of the OAuth 1.0 signature protocol, the need to get everything exactly 
byte-perfect, and the opacity of the products, there is surprisingly little example data out
there. Even the spec only gives an example using the `PLAINTEXT` "hash" algorithm. Which is 
useful, but getting a sample hash for known data for the other two algorithms in the spec would have
been even more helpful, since I doubt anyone is using `PLAINTEXT` in the wild. Especially since
OAuth 2.0 makes the use of OAuth 1.0 a questionable decision. (I'm looking at you, eTrade!)

I did find a couple of examples online:

Here is a [test case for OAuth 1.0](https://wiki.oauth.net/w/page/12238556/TestCases).
Here is [another one](https://lti.tools/oauth/).

[This isn't an "example,"](https://help.akana.com/content/current/cm/api_oauth/aaref/Ref_OAuth_AuthorizationHeader_10a.htm)
but it might be worth reading anyway.

This is an [article](https://www.testim.io/blog/how-to-test-oauth-authentication/) on how to secure your implementation.

# Desired API?

All I really want is to query eTrade for some live market data and my portfolio information. 
However, since it might be generally useful to have an OAuth 1.0 library available, I'm writing
this as a separate crate. (Although, who the heck is still using OAuth 1.0? Hopefully no one.)

I would like something like this:

```rust
  // In particular, it should:
  // - allow for auth config to be store in a config file
  // - store dynamic auth data into a secure file for reuse
  //   - check that the file is read-only, or spew a WARNING.
  let consumer = BasicConsumer::builder()
    .use_preset(ETradePreset)
    .use_secrets_file("<path to config file>")?
    .use_save_file("<path to writable config file>")?
    .build()?;

  let etrade = ETrade::builder()
    .use_consumer(consumer)
    .build()?;

  // Note that the auth handshake is hidden inside the consumer.
  // Consider putting the consumer behind the ETrade API boundary, too.
  let account_list = etrade.get_account_list()?;
  
  // and so on....
```
