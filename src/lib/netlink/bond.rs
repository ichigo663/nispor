use crate::netlink::nla::parse_as_u16;
use crate::netlink::nla::parse_as_u32;
use crate::netlink::nla::parse_as_u8;
use crate::parse_as_mac;
use crate::BondAdInfo;
use crate::BondInfo;
use crate::BondMiiStatus;
use crate::BondMode;
use crate::BondSubordinateInfo;
use crate::BondSubordinateState;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use std::net::Ipv4Addr;

const IFLA_BOND_MODE: u16 = 1;
const IFLA_BOND_AD_INFO: u16 = 23;

fn parse_as_nested_ipv4_addr(raw: &[u8]) -> Vec<Ipv4Addr> {
    let mut addresses = Vec::new();
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => {
                let data = nla.value();
                addresses
                    .push(Ipv4Addr::new(data[0], data[1], data[2], data[3]));
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    addresses
}

fn ipv4_addr_array_to_string(addrs: &[Ipv4Addr]) -> String {
    let mut rt = String::new();
    for i in 0..(addrs.len()) {
        rt.push_str(&addrs[i].to_string());
        if i != addrs.len() - 1 {
            rt.push_str(",");
        }
    }
    rt
}

fn parse_as_48_bits_mac(data: &[u8]) -> String {
    parse_as_mac(6, data)
}

const IFLA_BOND_AD_INFO_AGGREGATOR: u16 = 1;
const IFLA_BOND_AD_INFO_NUM_PORTS: u16 = 2;
const IFLA_BOND_AD_INFO_ACTOR_KEY: u16 = 3;
const IFLA_BOND_AD_INFO_PARTNER_KEY: u16 = 4;
const IFLA_BOND_AD_INFO_PARTNER_MAC: u16 = 5;

fn parse_ad_info(raw: &[u8]) -> BondAdInfo {
    let nlas = NlasIterator::new(raw);
    let mut ad_info = BondAdInfo::default();
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_AD_INFO_AGGREGATOR => {
                    ad_info.aggregator = parse_as_u16(nla.value());
                }
                IFLA_BOND_AD_INFO_NUM_PORTS => {
                    ad_info.num_ports = parse_as_u16(nla.value());
                }
                IFLA_BOND_AD_INFO_ACTOR_KEY => {
                    ad_info.actor_key = parse_as_u16(nla.value());
                }
                IFLA_BOND_AD_INFO_PARTNER_KEY => {
                    ad_info.partner_key = parse_as_u16(nla.value());
                }
                IFLA_BOND_AD_INFO_PARTNER_MAC => {
                    ad_info.partner_mac = parse_as_48_bits_mac(nla.value());
                }
                _ => {
                    eprintln!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    ad_info
}

fn get_bond_mode(raw: &[u8]) -> BondMode {
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_MODE => {
                    return parse_as_u8(nla.value()).into();
                }
                _ => (),
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    eprintln!("Failed to parse bond mode from NLAS: {:?}", nlas);
    BondMode::Unknown
}

// TODO: Use macro to generate function below
fn parse_miimon(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.miimon = Some(parse_as_u32(data));
}

fn parse_void(_data: &[u8], _bond_info: &mut BondInfo) {}

fn parse_updelay(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.updelay = Some(parse_as_u32(data));
}

fn parse_downdelay(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.downdelay = Some(parse_as_u32(data));
}

fn parse_use_carrier(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.use_carrier = Some(parse_as_u8(data) > 0);
}

fn parse_arp_interval(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.arp_interval = Some(parse_as_u32(data));
}

fn parse_arp_ip_target(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.arp_ip_target =
        Some(ipv4_addr_array_to_string(&parse_as_nested_ipv4_addr(data)));
}

fn parse_arp_all_targets(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.arp_all_targets = Some(parse_as_u32(data).into());
}

fn parse_arp_validate(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.arp_validate = Some(parse_as_u32(data).into());
}

fn parse_primary(data: &[u8], bond_info: &mut BondInfo) {
    if [
        BondMode::ActiveBackup,
        BondMode::BalanceAlb,
        BondMode::BalanceTlb,
    ]
    .contains(&bond_info.mode)
    {
        bond_info.primary = Some(format!("{}", parse_as_u32(data)));
    }
}

fn parse_primary_reselect(data: &[u8], bond_info: &mut BondInfo) {
    if [
        BondMode::ActiveBackup,
        BondMode::BalanceAlb,
        BondMode::BalanceTlb,
    ]
    .contains(&bond_info.mode)
    {
        bond_info.primary_reselect = Some(parse_as_u8(data).into());
    }
}

fn parse_fail_over_mac(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::ActiveBackup {
        bond_info.fail_over_mac = Some(parse_as_u8(data).into());
    }
}

fn parse_xmit_hash_policy(data: &[u8], bond_info: &mut BondInfo) {
    if [
        BondMode::BalanceXor,
        BondMode::Ieee8021AD,
        BondMode::BalanceTlb,
    ]
    .contains(&bond_info.mode)
    {
        bond_info.xmit_hash_policy = Some(parse_as_u8(data).into());
    }
}

fn parse_resend_igmp(data: &[u8], bond_info: &mut BondInfo) {
    if [
        BondMode::BalanceRoundRobin,
        BondMode::ActiveBackup,
        BondMode::BalanceTlb,
        BondMode::BalanceAlb,
    ]
    .contains(&bond_info.mode)
    {
        bond_info.resend_igmp = Some(parse_as_u32(data));
    }
}

fn parse_num_peer_notif(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::ActiveBackup {
        bond_info.num_unsol_na = Some(parse_as_u8(data));
        bond_info.num_grat_arp = Some(parse_as_u8(data));
    }
}

fn parse_all_subordinates_active(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.all_subordinates_active = Some(parse_as_u8(data).into());
}

fn parse_min_links(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.min_links = Some(parse_as_u32(data));
    }
}

fn parse_lp_interval(data: &[u8], bond_info: &mut BondInfo) {
    if [BondMode::BalanceTlb, BondMode::BalanceAlb].contains(&bond_info.mode) {
        bond_info.lp_interval = Some(parse_as_u32(data));
    }
}

fn parse_packets_per_subordinate(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::BalanceRoundRobin {
        bond_info.packets_per_subordinate = Some(parse_as_u32(data));
    }
}
fn parse_ad_lacp_rate(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.lacp_rate = Some(parse_as_u8(data).into());
    }
}

fn parse_ad_select(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.ad_select = Some(parse_as_u8(data).into());
    }
}

fn parse_ad_actor_sys_prio(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.ad_actor_sys_prio = Some(parse_as_u16(data));
    }
}

fn parse_ad_user_port_key(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.ad_user_port_key = Some(parse_as_u16(data));
    }
}

fn parse_ad_actor_system(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::Ieee8021AD {
        bond_info.ad_actor_system = Some(parse_as_48_bits_mac(data));
    }
}

fn parse_tlb_dynamic_lb(data: &[u8], bond_info: &mut BondInfo) {
    if bond_info.mode == BondMode::BalanceTlb {
        bond_info.tlb_dynamic_lb = Some(parse_as_u8(data) > 0);
    }
}

fn parse_peer_notif_delay(data: &[u8], bond_info: &mut BondInfo) {
    bond_info.peer_notif_delay = Some(parse_as_u32(data));
}

const NLA_PARSE_FUNS: &[fn(&[u8], &mut BondInfo)] = &[
    parse_void, // IFLA_BOND_UNSPEC
    parse_void, // IFLA_BOND_MODE parsed by get_bond_mode()
    parse_void, // IFLA_BOND_ACTIVE_SLAVE is deprecated
    parse_miimon,
    parse_updelay,
    parse_downdelay,
    parse_use_carrier,
    parse_arp_interval,
    parse_arp_ip_target,
    parse_arp_validate,
    parse_arp_all_targets,
    parse_primary,
    parse_primary_reselect,
    parse_fail_over_mac,
    parse_xmit_hash_policy,
    parse_resend_igmp,
    parse_num_peer_notif,
    parse_all_subordinates_active,
    parse_min_links,
    parse_lp_interval,
    parse_packets_per_subordinate,
    parse_ad_lacp_rate,
    parse_ad_select,
    parse_void, // IFLA_BOND_AD_INFO, handled by parse_ad_info().
    parse_ad_actor_sys_prio,
    parse_ad_user_port_key,
    parse_ad_actor_system,
    parse_tlb_dynamic_lb,
    parse_peer_notif_delay,
];

pub(crate) fn parse_bond_info(raw: &[u8]) -> BondInfo {
    let mut bond_info = BondInfo::default();
    bond_info.mode = get_bond_mode(raw);
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => {
                if let Some(func) =
                    NLA_PARSE_FUNS.get::<usize>(nla.kind().into())
                {
                    func(nla.value(), &mut bond_info);
                } else if nla.kind() == IFLA_BOND_AD_INFO {
                    bond_info.ad_info = Some(parse_ad_info(nla.value()));
                } else {
                    eprintln!(
                        "Failed to parse IFLA_LINKINFO for bond: {:?} {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to parse IFLA_LINKINFO {:?}", e);
            }
        }
    }
    bond_info
}

const IFLA_BOND_SLAVE_STATE: u16 = 1;
const IFLA_BOND_SLAVE_MII_STATUS: u16 = 2;
const IFLA_BOND_SLAVE_LINK_FAILURE_COUNT: u16 = 3;
const IFLA_BOND_SLAVE_PERM_HWADDR: u16 = 4;
const IFLA_BOND_SLAVE_QUEUE_ID: u16 = 5;
const IFLA_BOND_SLAVE_AD_AGGREGATOR_ID: u16 = 6;
const IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE: u16 = 7;
const IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE: u16 = 8;

pub(crate) fn parse_bond_subordinate_info(raw: &[u8]) -> BondSubordinateInfo {
    let nlas = NlasIterator::new(raw);
    let mut subordinate_state = BondSubordinateState::Unknown;
    let mut mii_status = BondMiiStatus::Unknown;
    let mut link_failure_count = std::u32::MAX;
    let mut perm_hwaddr = String::new();
    let mut queue_id = std::u16::MAX;
    let mut ad_aggregator_id = None;
    let mut ad_actor_oper_port_state = None;
    let mut ad_partner_oper_port_state = None;
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_SLAVE_STATE => {
                    subordinate_state = parse_as_u8(nla.value()).into()
                }
                IFLA_BOND_SLAVE_MII_STATUS => {
                    mii_status = parse_as_u8(nla.value()).into()
                }
                IFLA_BOND_SLAVE_LINK_FAILURE_COUNT => {
                    link_failure_count = parse_as_u32(nla.value())
                }
                IFLA_BOND_SLAVE_PERM_HWADDR => {
                    perm_hwaddr = parse_as_mac(nla.value_length(), nla.value());
                }
                IFLA_BOND_SLAVE_QUEUE_ID => {
                    queue_id = parse_as_u16(nla.value())
                }
                IFLA_BOND_SLAVE_AD_AGGREGATOR_ID => {
                    ad_aggregator_id = Some(parse_as_u16(nla.value()));
                }
                IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE => {
                    ad_actor_oper_port_state = Some(parse_as_u8(nla.value()));
                }
                IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE => {
                    ad_partner_oper_port_state =
                        Some(parse_as_u16(nla.value()));
                }
                _ => {
                    eprintln!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    BondSubordinateInfo {
        subordinate_state,
        mii_status,
        link_failure_count,
        perm_hwaddr,
        queue_id,
        ad_aggregator_id,
        ad_actor_oper_port_state,
        ad_partner_oper_port_state,
    }
}
