Given the complexity of this protocol's signature protocol, the need to get everything exactly 
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

# Spec "checklist"

As I address sections of the spec in my implementation (or decide I don't need to address them),
I am marking them here. 

~~1.  Authors~~  
~~2.  Notation and Conventions~~    
~~3.  Definitions~~  
~~4.  Documentation and Registration~~  
    ~~4.1.  Request URLs~~  
    ~~4.2.  Service Providers~~  
    ~~4.3.  Consumers~~  
5.  Parameters  
    ~~5.1.  Parameter Encoding~~  
    5.2.  Consumer Request Parameters  
    ~~5.3.  Service Provider Response Parameters~~  
    5.4.  OAuth HTTP Authorization Scheme  
6.  Authenticating with OAuth  
    6.1.  Obtaining an Unauthorized Request Token  
    6.2.  Obtaining User Authorization  
    6.3.  Obtaining an Access Token  
7.  Accessing Protected Resources  
~~8.  Nonce and Timestamp~~  
9.  Signing Requests  
    ~~9.1.  Signature Base String~~  
    9.2.  HMAC-SHA1  
    ~~9.3.  RSA-SHA1~~  
    ~~9.4.  PLAINTEXT~~  
10.  HTTP Response Codes  
11.  Security Considerations
     11.1.  Credentials and Token Exchange
     11.2.  PLAINTEXT Signature Method
     11.3.  Confidentiality of Requests
     11.4.  Spoofing by Counterfeit Servers
     11.5.  Proxying and Caching of Authenticated Content
     11.6.  Plaintext Storage of Credentials
     11.7.  Secrecy of the Consumer Secret
     11.8.  Phishing Attacks
     11.9.  Scoping of Access Requests
     11.10.  Entropy of Secrets
     11.11.  Denial of Service / Resource Exhaustion Attacks
     11.12.  Cryptographic Attacks
     11.13.  Signature Base String Compatibility
     11.14.  Cross-Site Request Forgery (CSRF)
     11.15.  User Interface Redress
     11.16.  Automatic Processing of Repeat Authorizations
     Appendix A.  Appendix A - Protocol Example
     Appendix A.1.  Documentation and Registration
     Appendix A.2.  Obtaining a Request Token
     Appendix A.3.  Requesting User Authorizatio