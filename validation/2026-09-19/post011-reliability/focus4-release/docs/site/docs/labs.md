---
id: labs
title: "Labs and answer keys"
sidebar_label: "Labs and answer keys"
---

## Six tabletop exercises

The lessons so far have asked the same two questions at every arrow: what crosses this boundary, and what authorizes the receiver to act? These exercises check whether you can answer them without the lesson text in front of you. Each one takes a situation the course has already explained and changes one condition, so the work is in noticing which condition decides the outcome.

Use fictional identities and inputs throughout. None of the six exercises needs a production account or a cloud API credential; a whiteboard and the running example are enough.

### Lab 1: Trace two requests

Draw the harness, Keycloak, CIE/CAS, AI Gateway, the model and mcp server 1. Then trace Alex's question: "Multiply 12 by 7 and return the MCP server's UTC time."

**Deliverable:** two labeled request paths and the execution location of each tool.

**Answer key:** there are two paths, and both end at the gateway before going anywhere else. Inference goes from the harness through the gateway to the model. MCP goes from the native client through the gateway to mcp server 1. The server computes the product and reads its own clock, so the execution location for both tools is the server, not the workstation and not the model. Results return through the gateway. The deciding detail is who holds which credential: the gateway holds the upstream OAuth credentials, and the harness never sees them.

### Lab 2: Evaluate the utility grant

Assume a correctly signed, current token for the configured issuer and MCP audience unless a row says otherwise. For each case, decide what mcp server 1 does with the request.

| Case | Conditions | Action |
| --- | --- | --- |
| A | Allowed gateway client, invoke, utilities.use scope and role, matching subject policy | calculate |
| B | Same grants as A, but an inference audience | calculate |
| C | Same as A, but utilities.use is missing from the token | hash_text |
| D | Same as A, but the utilities.use role belongs to another resource client | format_json |
| E | Same as A, but no subject binding | current_time |
| F | Same as A, but the client is not allowed | generate_uuid |
| G | All grants from A, arguments request division by zero | calculate |

**Deliverable:** distinguish authentication denial, authorization denial, successful execution and input failure.

**Answer key:** the server checks these conditions in order, and the first failing check names the category. A passes every check and executes. B and F fail token or client validation before any grant is considered: B was issued for the wrong audience, and F arrived from a client the server does not allow. C, D and E fail authorization, because effective access is the intersection of the issued scope, the resource-specific role and the subject binding, and each row removes a different one of those three. G passes every identity and authorization check and then fails on its arguments, so it returns an input error. That is why login is not a repair for G: nothing about the credential was wrong.

### Lab 3: Separate directory lookup from login

CIE has `alex@example.com`. CAS accepts a SAML assertion whose configured username field contains `alex`. The gateway cannot resolve a provisioned user. Another realm also contains the email `alex@example.com`.

**Deliverable:** name the mapping to inspect and explain why matching emails do not establish one identity.

**Answer key:** the login succeeded and the directory record exists, so the failure is in the join between them. Compare the SAML attribute the gateway actually consumes (including NameID if that is the configured field) with the field the provisioned record is looked up by; here one side carries `alex` and the other carries `alex@example.com`. The second realm is the reason an email match cannot be the fix: an email is a lookup attribute, and two issuers can hold the same email for unrelated subjects, so the durable identity stays issuer plus subject. After the user resolves, verify gateway workspace membership separately, because resolving an identity and being a member of the intended workspace are different facts.

### Lab 4: Interrupt rotating refresh

Two native processes share a credential binding. Process A retires refresh generation 4, receives generation 5, and crashes before saving it.

**Deliverable:** show the lock, the pending state and the next user's recovery path.

**Answer key:** only one process exchanges a generation at a time, which is what the binding lock is for. Process A recorded its refresh intent before the exchange, so when the next process acquires the lock it finds unresolved state rather than a clean generation 4. It must request sign-in instead of replaying generation 4, because that refresh token may already have been consumed by the exchange that produced generation 5. The deciding fact is that rotation may already have consumed generation 4, so an uncertain exchange cannot be retried safely. In the normal case, generation 5 is saved before it is ever used, and the second process simply reloads it.

### Lab 5: State what utility output proves

`calculate` returns 84 and `current_time` returns a timestamp. The model says it has inspected the gateway's security profile and verified scanning.

**Deliverable:** replace that answer with two defensible sentences.

**Answer key:** "mcp server 1 returned 84 for 12 × 7. Its clock reported the displayed UTC timestamp." Each sentence claims only what a completed tool result establishes. Neither utility inspected a security profile or returned a scanner verdict, so the model's original statement asserts evidence that no tool in this inventory can produce.

### Lab 6: Reason about idle return and revocation

A gateway MCP access token remains valid for another 30 minutes, but the upstream user grant has exceeded its idle limit. Separately, an operator removes the user's binding from mcp server 1.

**Deliverable:** explain why a valid gateway credential may still be insufficient.

**Answer key:** the gateway credential only gets the request as far as the gateway. To proxy the call, the gateway also needs a usable upstream grant for this user, and that grant expired on its own clock; renewing it may require consent again. The binding removal is a third clock: once every server replica has loaded the new policy, that subject is denied even if it presents an unexpired upstream JWT, because the binding is one of the server's own authorization checks rather than a claim inside the token. CIE workspace deprovisioning would propagate along yet another path. The reason one valid token is not enough is that each receiver enforces its own contract, and those contracts expire and revoke independently.

## Optional isolated integration exercise

The tabletop labs test the model. This exercise tests it against a real gateway route, and it only makes sense in an isolated environment: use a maintainer-provided gateway route, test identity, workspace mapping and utility-server binding. The exercise needs no management-API service account, because the utility tools do not call one. Native credentials still require the appropriate OS store, since that is where the harness keeps them.

| Step | Observation to record |
| --- | --- |
| Request gateway MCP without a credential | OAuth challenge from the intended gateway |
| Complete gateway and required upstream login | Native save succeeds; correct identity and integration selected |
| Discover tools | Exactly the expected eight utility names |
| Call each tool with synthetic valid input | Completed result with the expected type and content |
| Submit invalid arithmetic, JSON and encoding inputs | Bounded tool errors without losing authorization |
| Test missing scope, role or binding in the isolated fixture | Denial at the intended boundary |
| Continue real activity across token expiry | Actual utility success with renewed credentials |
| Return after the preserved idle timeout | Clear sign-in guidance and verified restoration behavior |
| Finish and remove the fixture | Local credentials and test grants removed without disrupting another test |

Two operational cautions follow from the lifecycle lesson. Use dedicated fixture bindings and coordinate issuer logout when tests share a browser SSO session, because inference logout can revoke the client session that another login is relying on. And do not turn a quiet terminal into a keepalive test: idle expiry is an intended policy outcome, so the test should observe it rather than prevent it.

Score the explanation on request routing, execution location, credential separation, the complete utility grant and the limits of the evidence. Keep the last of those honest: a successful utility call shows that the path worked once, and it does not by itself prove timed renewal.

Review [Architecture](./architecture.md), [Utility authorization](./mcp.md) and [Credential lifecycle](./lifecycle.md).
