//! Registry of retailer sourcing configurations.

use crate::retailers::{
    // anker::{my as ankermy},
    anker::{
        au as ankerau, ca as ankerca, com as ankercom, de as ankerde, eu as ankereu, fr as ankerfr,
        italycom as ankeritalycom, japancom as ankerjapancom, kr as ankerkr, nordicscom as ankernordicscom,
        nz as ankernz, pl as ankerpl, uk as ankeruk, vn as ankervn,
    },
    micom,
    minisforum::{
        au as minisforumau, ca as minisforumca, eu as minisforumeu, fr as minisforumfr, hk as minisforumhk,
        jp as minisforumjp, kr as minisforumkr, ru as minisforumru, uk as minisforumuk, us as minisforumus,
    },
    ugreen::{
        ca as ugreenca, com as ugreencom, de as ugreende, eu as ugreeneu, fr as ugreenfr, r#in as ugreenin,
        jp as ugreenjp, kr as ugreenkr, nas as ugreennas, nasau as ugreennasau, nasca as ugreennasca,
        nasde as ugreennasde, nases as ugreennases, naseu as ugreennaseu, nasfr as ugreennasfr, nasit as ugreennasit,
        nasjp as ugreennasjp, nasuk as ugreennasuk, nl as ugreennl, uk as ugreenuk, us as ugreenus,
    },
};

use bodul_shared::SitemapConfig;
use bodul_shared::link::LinkKind;
use bodul_shared::retailer::RetailerCode;

/// Returns the offer-detail architecture for a retailer.
pub fn architecture_for(retailer: RetailerCode) -> crate::parsing::structure::RetailerArchitecture {
    match retailer {
        RetailerCode::MinisForumAu => minisforumau::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumCa => minisforumca::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumEu => minisforumeu::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumUs => minisforumus::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumUk => minisforumuk::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumFr => minisforumfr::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumKr => minisforumkr::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumJp => minisforumjp::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumRu => minisforumru::prelude::offer_detail_architecture_v1(),
        RetailerCode::MinisForumHk => minisforumhk::prelude::offer_detail_architecture_v1(),
        RetailerCode::MiCom => micom::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenCom => ugreencom::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenUs => ugreenus::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenCa => ugreenca::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenEu => ugreeneu::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenDe => ugreende::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenUk => ugreenuk::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenFr => ugreenfr::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNl => ugreennl::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenJp => ugreenjp::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenKr => ugreenkr::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenIn => ugreenin::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNas => ugreennas::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasCa => ugreennasca::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasEu => ugreennaseu::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasDe => ugreennasde::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasUk => ugreennasuk::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasFr => ugreennasfr::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasEs => ugreennases::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasIt => ugreennasit::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasAu => ugreennasau::prelude::offer_detail_architecture_v1(),
        RetailerCode::UgreenNasJp => ugreennasjp::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerCom => ankercom::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerJapanCom => ankerjapancom::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerKr => ankerkr::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerItalyCom => ankeritalycom::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerNordicsCom => ankernordicscom::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerUk => ankeruk::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerCa => ankerca::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerEu => ankereu::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerDe => ankerde::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerFr => ankerfr::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerPl => ankerpl::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerAu => ankerau::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerNz => ankernz::prelude::offer_detail_architecture_v1(),
        // RetailerCode::AnkerMy => ankermy::prelude::offer_detail_architecture_v1(),
        RetailerCode::AnkerVn => ankervn::prelude::offer_detail_architecture_v1(),
    }
}

/// Resolves a retailer's sitemap configuration.
///
/// Returns `None` when the retailer has no known sitemap URLs.
pub fn sitemap_config(code: &RetailerCode) -> Option<SitemapConfig> {
    let config = match code {
        RetailerCode::MinisForumEu => minisforumeu::prelude::sitemap_config(),
        RetailerCode::MinisForumUs => minisforumus::prelude::sitemap_config(),
        RetailerCode::MinisForumUk => minisforumuk::prelude::sitemap_config(),
        RetailerCode::MinisForumFr => minisforumfr::prelude::sitemap_config(),
        RetailerCode::MinisForumCa => minisforumca::prelude::sitemap_config(),
        RetailerCode::MinisForumAu => minisforumau::prelude::sitemap_config(),
        RetailerCode::MinisForumKr => minisforumkr::prelude::sitemap_config(),
        RetailerCode::MinisForumJp => minisforumjp::prelude::sitemap_config(),
        RetailerCode::MinisForumRu => minisforumru::prelude::sitemap_config(),
        RetailerCode::MinisForumHk => minisforumhk::prelude::sitemap_config(),
        RetailerCode::MiCom => micom::prelude::sitemap_config(),
        RetailerCode::UgreenCom => ugreencom::prelude::sitemap_config(),
        RetailerCode::UgreenUs => ugreenus::prelude::sitemap_config(),
        RetailerCode::UgreenCa => ugreenca::prelude::sitemap_config(),
        RetailerCode::UgreenEu => ugreeneu::prelude::sitemap_config(),
        RetailerCode::UgreenDe => ugreende::prelude::sitemap_config(),
        RetailerCode::UgreenUk => ugreenuk::prelude::sitemap_config(),
        RetailerCode::UgreenFr => ugreenfr::prelude::sitemap_config(),
        RetailerCode::UgreenNl => ugreennl::prelude::sitemap_config(),
        RetailerCode::UgreenJp => ugreenjp::prelude::sitemap_config(),
        RetailerCode::UgreenKr => ugreenkr::prelude::sitemap_config(),
        RetailerCode::UgreenIn => ugreenin::prelude::sitemap_config(),
        RetailerCode::UgreenNas => ugreennas::prelude::sitemap_config(),
        RetailerCode::UgreenNasCa => ugreennasca::prelude::sitemap_config(),
        RetailerCode::UgreenNasEu => ugreennaseu::prelude::sitemap_config(),
        RetailerCode::UgreenNasDe => ugreennasde::prelude::sitemap_config(),
        RetailerCode::UgreenNasUk => ugreennasuk::prelude::sitemap_config(),
        RetailerCode::UgreenNasFr => ugreennasfr::prelude::sitemap_config(),
        RetailerCode::UgreenNasEs => ugreennases::prelude::sitemap_config(),
        RetailerCode::UgreenNasIt => ugreennasit::prelude::sitemap_config(),
        RetailerCode::UgreenNasAu => ugreennasau::prelude::sitemap_config(),
        RetailerCode::UgreenNasJp => ugreennasjp::prelude::sitemap_config(),
        RetailerCode::AnkerCom => ankercom::prelude::sitemap_config(),
        RetailerCode::AnkerJapanCom => ankerjapancom::prelude::sitemap_config(),
        RetailerCode::AnkerKr => ankerkr::prelude::sitemap_config(),
        RetailerCode::AnkerItalyCom => ankeritalycom::prelude::sitemap_config(),
        RetailerCode::AnkerNordicsCom => ankernordicscom::prelude::sitemap_config(),
        RetailerCode::AnkerUk => ankeruk::prelude::sitemap_config(),
        RetailerCode::AnkerCa => ankerca::prelude::sitemap_config(),
        RetailerCode::AnkerEu => ankereu::prelude::sitemap_config(),
        RetailerCode::AnkerDe => ankerde::prelude::sitemap_config(),
        RetailerCode::AnkerFr => ankerfr::prelude::sitemap_config(),
        RetailerCode::AnkerPl => ankerpl::prelude::sitemap_config(),
        RetailerCode::AnkerAu => ankerau::prelude::sitemap_config(),
        RetailerCode::AnkerNz => ankernz::prelude::sitemap_config(),
        // RetailerCode::AnkerMy => ankermy::prelude::sitemap_config(),
        RetailerCode::AnkerVn => ankervn::prelude::sitemap_config(),
    };

    if config.sitemap_url.is_empty() {
        return None;
    }

    Some(config)
}

/// Classifies a page URL using `code`'s retailer-specific rules.
///
/// Each retailer's rule lives alongside its `sitemap_config`, exposed as a
/// `classify_link` fn in its module. Retailers without a rule yet fall
/// through to [`LinkKind::Unknown`].
pub fn classify_link(code: RetailerCode, url: &str, source: &str, image_count: usize) -> LinkKind {
    match code {
        RetailerCode::MinisForumEu => minisforumeu::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumUs => minisforumus::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumUk => minisforumuk::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumFr => minisforumfr::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumCa => minisforumca::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumAu => minisforumau::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumKr => minisforumkr::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumJp => minisforumjp::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumRu => minisforumru::prelude::classify_link(url, source, image_count),
        RetailerCode::MinisForumHk => minisforumhk::prelude::classify_link(url, source, image_count),
        RetailerCode::MiCom => micom::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenCom => ugreencom::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenUs => ugreenus::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenCa => ugreenca::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenEu => ugreeneu::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenDe => ugreende::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenUk => ugreenuk::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenFr => ugreenfr::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNl => ugreennl::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenJp => ugreenjp::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenKr => ugreenkr::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenIn => ugreenin::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNas => ugreennas::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasCa => ugreennasca::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasEu => ugreennaseu::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasDe => ugreennasde::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasUk => ugreennasuk::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasFr => ugreennasfr::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasEs => ugreennases::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasIt => ugreennasit::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasAu => ugreennasau::prelude::classify_link(url, source, image_count),
        RetailerCode::UgreenNasJp => ugreennasjp::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerCom => ankercom::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerJapanCom => ankerjapancom::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerKr => ankerkr::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerItalyCom => ankeritalycom::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerNordicsCom => ankernordicscom::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerUk => ankeruk::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerCa => ankerca::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerEu => ankereu::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerDe => ankerde::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerFr => ankerfr::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerPl => ankerpl::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerAu => ankerau::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerNz => ankernz::prelude::classify_link(url, source, image_count),
        // Doesnt exist anymore, it redirect to lazada.com webshop
        // RetailerCode::AnkerMy => ankermy::prelude::classify_link(url, source, image_count),
        RetailerCode::AnkerVn => ankervn::prelude::classify_link(url, source, image_count),
    }
}

/// The environment variable holding a retailer's session cookie, if it needs one.
///
/// None of the currently supported retailers require a session cookie.
pub fn cookie_env_var(_code: RetailerCode) -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    use super::{architecture_for, classify_link, cookie_env_var, sitemap_config};
    use bodul_shared::link::LinkKind;
    use bodul_shared::retailer::RetailerCode;

    #[test]
    fn resolves_configuration_for_every_supported_retailer() {
        for code in RetailerCode::ALL {
            // Some registered architectures are intentionally empty scaffolds.
            let _architecture = architecture_for(code);

            let config = sitemap_config(&code).expect("supported retailer must have a sitemap");
            assert!(!config.sitemap_url.is_empty(), "missing sitemap URLs for {code:?}");
            assert_eq!(cookie_env_var(code), None, "unexpected cookie requirement for {code:?}");
        }
    }

    #[test]
    fn classifies_links_through_every_supported_retailer() {
        let cases = [
            ("/products/example", LinkKind::Product),
            ("/PRODUCTS/EXAMPLE", LinkKind::Product),
            ("/collections/all", LinkKind::Catalog),
            ("/pages/about", LinkKind::Content),
            ("/blogs/news/example", LinkKind::Content),
            ("/", LinkKind::Unknown),
        ];

        for code in RetailerCode::ALL {
            let config = sitemap_config(&code).expect("supported retailer must have a sitemap");
            let (site, _) = config.sitemap_url[0]
                .rsplit_once('/')
                .expect("sitemap must have a URL path");
            for (path, expected) in &cases {
                let url = format!("{site}{path}");
                assert_eq!(classify_link(code, &url, "", 0), *expected, "for {code:?}: {url}");
            }
        }
    }
}
