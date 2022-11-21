(function() {var implementors = {
"jormungandr":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"jormungandr/intercom/struct.ReplyFuture.html\" title=\"struct jormungandr::intercom::ReplyFuture\">ReplyFuture</a>&lt;T&gt;"],["impl&lt;T, E&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"jormungandr/intercom/struct.ReplyStreamFuture.html\" title=\"struct jormungandr::intercom::ReplyStreamFuture\">ReplyStreamFuture</a>&lt;T, E&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"jormungandr/intercom/struct.Error.html\" title=\"struct jormungandr::intercom::Error\">Error</a>&gt;,</span>"],["impl&lt;TID, WID, Data, Launcher&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"jormungandr/utils/fire_forget_scheduler/struct.FireForgetSchedulerFuture.html\" title=\"struct jormungandr::utils::fire_forget_scheduler::FireForgetSchedulerFuture\">FireForgetSchedulerFuture</a>&lt;TID, WID, Data, Launcher&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;TID: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;WID: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Data: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Launcher: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/ops/function/trait.Fn.html\" title=\"trait core::ops::function::Fn\">Fn</a>(TID, WID, Data) + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,</span>"]],
"settings":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.65.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"settings/struct.Subscriber.html\" title=\"struct settings::Subscriber\">Subscriber</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()