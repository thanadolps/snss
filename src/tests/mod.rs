use super::*;

#[test]
fn test_parse() {
    let data = include_bytes!("Session");

    let snss = parse(data.as_slice()).unwrap();

    assert_eq!(snss.version, 3);
    let [cmd1, cmd2, cmd3] = snss.commands.try_into().unwrap();

    assert_eq!(cmd1.id, 14);
    let Content::Other(c1) = cmd1.content else {
        panic!()
    };
    assert_eq!(c1.len(), 24);

    assert_eq!(cmd2.id, 6);
    let Content::Tab(c2) = cmd2.content else {
        panic!()
    };

    assert_eq!(c2.id, 1994883225);
    assert_eq!(c2.index, 0);
    assert_eq!(
        c2.url,
        "https://console.hetzner.cloud/projects/3687808/servers/64199561/graphs"
    );
    assert_eq!(c2.title, "primary · Hetzner Cloud");
    assert!(!c2.post);
    assert_eq!(c2.referrer_url, "https://console.hetzner.cloud/");
    assert_eq!(c2.reference_policy, 2);
    assert_eq!(
        c2.original_request_url,
        "https://console.hetzner.cloud/projects/3687808/servers/64199561/backup"
    );
    assert!(!c2.user_agent);
    assert_eq!(c2.transition.kind().unwrap(), PageTransitionType::Reload);
    assert_eq!(
        c2.transition.qualifiers(),
        PageTransitionQualifiers {
            back_forward: false,
            address_bar: false,
            homepage: true,
            chain_start: true,
            redirect_chain_end: true,
            client_redirect: true,
            server_redirect: true,
        }
    );

    assert_eq!(cmd3.id, 6);
    let Content::Tab(c3) = cmd3.content else {
        panic!()
    };

    assert_eq!(c3.id, 1994883225);
    assert_eq!(c3.index, 1);
    assert_eq!(
        c3.url,
        "https://console.hetzner.cloud/projects/3687808/servers/64199561/loadbalancers"
    );
    assert_eq!(c3.title, "primary · Hetzner Cloud");
    assert!(!c3.post);
    assert_eq!(c3.referrer_url, "https://console.hetzner.cloud/");
    assert_eq!(c3.reference_policy, 2);
    assert_eq!(
        c3.original_request_url,
        "https://console.hetzner.cloud/projects/3687808/servers/64199561/graphs"
    );
    assert!(!c3.user_agent);
    assert_eq!(c3.transition.kind().unwrap(), PageTransitionType::Reload);
    assert_eq!(
        c3.transition.qualifiers(),
        PageTransitionQualifiers {
            back_forward: false,
            address_bar: false,
            homepage: true,
            chain_start: true,
            redirect_chain_end: true,
            client_redirect: true,
            server_redirect: true,
        }
    );
}
