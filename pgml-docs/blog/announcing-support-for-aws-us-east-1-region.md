# Announcing Support for AWS us-east-1 Region



Lev Kokotov

August 8, 2023

Since we released PostgresML Cloud a few months ago, we've been exclusively operating out of the AWS Oregon data center. Some say that the West Coast is the Best Coast, but we firmly believe your database should be as close to your application as possible. Today, we are happy to announce that we've added support for the `us-east-1` AWS region, also known as N. Virginia, or the home base of most startups and half the websites you use on a daily basis.

## Impact

If you've been using our Oregon (`us-west-2`) deployments and decide to switch to `us-east-1` instead, you should be able to see a reduction in latency of up to 80ms. Typical latency between the two coasts, measured with simple pings, isn't very high, but when TCP is used, especially with encryption, every millisecond of additional round trip time is amplified.

To demonstrate the impact of moving the data closer to your application, we've created two PostgresML deployments: one on the East Coast and one on the West Coast. We then ran `pgbench` from a virtual machine in New York against both deployments. The results speak for themselves.

\


\


## Using the New Region

To take advantage of latency savings, you can [deploy a dedicated PostgresML database](https://postgresml.org/signup) in `us-east-1` today. We make it as simple as filling out a very short form and clicking "Create database".

## Performance is Key

At PostgresML, we care about performance above almost anything else. Bringing machine learning to the data layer allowed us to remove a major latency bottleneck experienced in typical ML applications, but that's only one part of the story. Bringing PostgresML as close as possible to your application is just as important. We've built our cloud to be region agnostic, and we'll continue to add support for more regions and cloud providers.
