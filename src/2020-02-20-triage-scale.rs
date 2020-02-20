/** # Triage at scale for the Rust team

<sup>*by [David Tolnay]&#8202;,&ensp;2020.20.02*</sup>

[David Tolnay]: https://github.com/dtolnay

<br>

Yesterday Mark-Simulacrum [announced] an experimental new notification-tracking
mechanism implemented in Triagebot, the bot run by the Rust infrastructure team
to perform issue assignment and labelling across the Rust project's GitHub
repositories.

I have been working with Mark on designing and iterating on the notification
system and wanted to write down my perspective on several ways that this work is
important for Rust's growth.

[announced]: https://internals.rust-lang.org/t/triagebot-notifications/11857?u=dtolnay

<br>

## Design summary

Triagebot's notifications are designed to be a *publicly readable*, *publicly
editable*, *roughly ordered* todo list for every team member. [This short
wiki][wiki] explains the details. By publicly editable, we mean that anyone
with a registered zulip-id can make modifications to any team member's list,
*and should feel free to do so*.

[wiki]: https://github.com/rust-lang/triagebot/wiki/Notifications

<br>

## Target audience 1: the Rust community

Of the four audiences and major benefits of our approach that I'll cover, I am
starting with this because it's the one I find the most essential and exciting.

I work on the standard library team and I run many widely used open source Rust
library projects on the side. With the massive growth of the Rust community and
all of these projects, [the amount of email I get is out of control][email] and
it's often no longer possible to read much less respond to everything going on.
This is frustrating to the constantly growing set of people who want questions
answered, bugs fixed, and PRs reviewed.

[email]: https://twitter.com/davidtolnay/status/1224054074514919424

With Triagebot notifications, for the first time we can empower the community to
participate actively in the triage process, for official rust-lang repos as well
as beyond. This is more productive than repeatedly pinging issues for attention.
The community is granted visibility into what the maintainers and team members
are busy with, and in return we get to ask that they make a best-effort honest
determination of how the item for which they want attention ranks against the
other things going on.

I strongly encourage people to take advantage of this to add or reorder items on
my list! I can use the help keeping track of where my attention is most needed.

<br>

## Target audience 2: triage working group

The folks doing official triage on the Rust repo now have a way to see what is
on each team member's plate and make intelligent load-balancing decisions.

Previously, if a PR sat for a while without review, the state of the art was for
triage to make a shot in the dark to assign it to a random other member of the
reviewing team, which is not necessarily productive.

<br>

## Target audience 3: contributors

Community members who send PRs or make issues now have something to track while
waiting on feedback.

One of the nightmares to me as a maintainer is the experience Jane describes in
[this meetup talk][talk]:

[talk]: https://www.youtube.com/watch?v=QKbdBwjra5o&feature=youtu.be&t=432

> *I remember distinctly one time leaving a comment on an issue very shyly being
> like "hey would it be okay if I take this issue?" and waiting two days for a
> response and overthinking it and freaking out, and eventually I just deleted
> the comment and tried to find somewhere else because I was too stressed about
> it.*

I know that I take two days off sometimes, or sometimes have urgent work that
takes away from lower priority projects. I of course wouldn't want someone to
have the above experience during those times.

Imagine if the comment that @rust-highfive leaves on new PRs included a link to
the reviewer's triagebot page. This gives a place for the PR author to track as
the reviewer makes progress toward the relevant PR, and a clear way to
internalize that there is a lot going on across the project &ndash; the lack of
immediate attention isn't because the PR is stupid, or the author is a minority,
or whatever other overthought reason. This makes for a friendlier environment
than waiting in a void.

<br>

## Target audience 4: Rust team members

I am excited for the community to help surface where my attention is needed
without me juggling zillions of emails. Email is no longer a reasonable way for
me to track and manage actionable notifications and I expect as Rust's momentum
grows that most other team members have already experienced the same or will
soon.

Separately, I hope that the design we're pursuing for Triagebot notifications
makes it easier for team members to avoid some common burnout traps that come
with participating in an exponentially scaling project over the next several
years. In a young, small, or slow-moving project, often the total work is seen
as a fixed quantity and a developer or maintainer will optimize for how quickly
they can accomplish the total work (implement all the features, respond to all
the issues, fix all the bugs, whatever it may be). But as projects get big or
important, in my experience participants commonly have trouble making the
transition to a mindset of treating their available volunteer time as the fixed
quantity, optimizing for how to make best use of that time without necessarily
regard for accomplishing "all" the work.

We address this in three ways: public write access is designed to surface
high-value actionable work without a team member needing to remain on top of a
huge volume of discussions; public read access helps the community build empathy
with the workload that team members are faced with; and the ordered nature
exposes a comfortable way to say no to lower impact work.

<br><br>

It's very early days for this system so far, but we'd love for people to kick
the tires and provide feedback of any kind! Check out [Mark's
announcement][announced] and the [Triagebot wiki][wiki] for additional details.
*/
#[macro_export]
macro_rules! _04__triage_scale {
    ({
        date:  "Feburary 20, 2020",
        author:  "David Tolnay",
    }) => {};
}
