//! HTML parsing.
//!
//! Extracts product and offer fields from stored product-page HTML using
//! retailer-specific rules (roadmap Stage E). Consumed by `process_products`.
//!
//! Each retailer defines a [`RetailerArchitecture`] — a tree of [`Structure`]
//! definitions built with [`structure::particle`], [`structure::collection`] and
//! [`structure::segment`]. The same architecture drives different consumers (the
//! `destructure` and `valueless` binaries); [`architecture_for`] returns it and
//! [`destructure`] applies it to extract values.

use bodul_shared::retailer::RetailerCode;
use kuchiki::traits::*;
use retailer_sourcing::architecture_for;

mod blank;
mod extract;
mod feature_chart;

pub use ::retailer_sourcing::parsing::structure::{
    Attribute, Collection, Json, Particle, RetailerArchitecture, Scrub, Segment, Structure, Trash, collection, json,
    json_after, particle, scrub, segment, trash,
};

/// Parses `html` and extracts `retailer`'s architecture into a JSON tree.
pub fn destructure(html: &str, retailer: RetailerCode) -> serde_json::Value {
    let document = kuchiki::parse_html().one(html);
    let mut value = extract::extract(&document, &architecture_for(retailer));
    feature_chart::transpose(&mut value);
    value
}

/// The blanked outputs of one page.
pub struct Valueless {
    /// The whole page, blanked.
    pub page: String,
    /// Each lifted top-level segment's blanked HTML, as `(name, html)`.
    pub sections: Vec<(String, String)>,
    /// Each lifted collection item, as `(collection_name, index, html)`.
    pub components: Vec<(String, usize, String)>,
}

/// Parses `html`, blanks it with `retailer`'s architecture, and returns the page,
/// each top-level section's HTML, and every lifted collection [`component`].
///
/// Top-level segments are lifted: each is replaced in the page by a `[name]`
/// placeholder and returned separately. Collections are componentized: each item
/// is replaced in the page by a `[name_index]` placeholder and returned separately.
pub fn valueless(html: &str, retailer: RetailerCode) -> Result<Valueless, std::io::Error> {
    let document = kuchiki::parse_html().one(html);
    let architecture = architecture_for(retailer);
    let blanked = blank::apply(&document, &architecture);

    let page = serialize_node(&document)?;

    let mut sections = Vec::new();
    for section in blanked.sections {
        let html = serialize_node(&section.node)?;
        sections.push((section.name, html));
    }

    let mut components = Vec::new();
    for component in blanked.components {
        components.push((component.name, component.index, serialize_node(&component.node)?));
    }

    Ok(Valueless {
        page,
        sections,
        components,
    })
}

/// Serializes a node (including itself) to an HTML string.
fn serialize_node(node: &kuchiki::NodeRef) -> Result<String, std::io::Error> {
    let mut out = Vec::new();
    node.serialize(&mut out)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

#[cfg(test)]
mod tests {
    use super::destructure;
    use bodul_shared::retailer::RetailerCode;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    const MATHEMA_SNIPPET: &str = r#"<!doctype html>
<html lang="hr" data-token="tok" data-current-lang="hr" data-currency_code="EUR">
<head>
  <title>Generic Product</title>
  <meta property="og:title" content="Generic Product">
  <meta property="og:description" content="Opis proizvoda">
  <meta property="og:url" content="https://example.com/generic-product/123/product/">
  <meta property="og:type" content="product">
  <meta property="product:price:amount" content="123.45">
  <meta property="product:price:currency" content="EUR">
  <meta property="product:brand" content="Brand">
  <meta property="product:availability" content="instock">
  <link rel="canonical" href="https://example.com/generic-product/123/product/">
  <script type="application/ld+json">
    {"@context":"https://schema.org","@type":"Product","sku":"SKU-123","productID":"123","brand":{"name":"Brand"},"offers":{"price":"123.45","priceCurrency":"EUR","availability":"http://schema.org/InStock","seller":{"name":"Seller"}}}
  </script>
  <script>
    window.dataLayer = window.dataLayer || [];
    window.dataLayer.push({ ecommerce: null });
    window.dataLayer.push({ 'event': 'view_item', 'ecommerce': { 'value': 123.45, 'currency': 'EUR', 'items': [{ 'item_name': 'Generic Product', 'item_id': '123', 'price': 123.45, 'item_category': 'Category', 'item_category2': 'Subcategory', 'quantity': 1, 'item_list_name': 'Product detail page' }] } });
  </script>
</head>
<body>
  <header id="top"></header>
  <nav class="wsmenu"></nav>
  <main>
    <div class="breadcrumb">
      <a href="/"><span>Naslovna</span></a>
      <a href="/category/"><span>Category</span></a>
    </div>
    <div class="productEntityDetailPage"
         id="item_123"
         data-product_name="Generic Product"
         data-product_id="123"
         data-product_category="Category"
         data-product_category2="Subcategory"
         data-productlist_position="1"
         data-productlist_name="Product detail page"
         data-product_price="123.45"
         data-product_price_before_discount="0"
         data-product_price_percentage_discount="0"
         data-product_discount="0"
         data-product_value="123.45"
         data-item_list_name="Product detail page"
         data-product_brand="Brand"
         data-showadd="1">
      <h1 class="c-title">Generic Product</h1>
      <div class="manufacture"><a href="/brand/1/">Brand</a></div>
      <div class="sifra">Šifra: <span>SKU-123</span></div>
      <div class="barcode">Barkod: <span>3850000001234</span></div>
      <div class="availability">Dostupnost artikla: <span>Na stanju</span></div>
      <div class="price"><span class="mainprice singleprice standard-price">123,45 EUR</span></div>
      <div class="opis opisproduct">Opis proizvoda</div>
      <button class="productEntityAddToCart" data-id="SKU-123" data-name="Generic Product" data-category="Category" data-price="123.45"></button>
      <div id="big"><div class="item"><a class="fancybox" href="/images/a.webp"><img src="/images/a.webp" alt="Generic Product"></a></div></div>
    </div>
    <form id="upit_info" action="/generic-product/123/product/?handler=Upit"><input id="upit_product_id" value="123"></form>
  </main>
  <footer class="footer"></footer>
</body>
</html>"#;

    macro_rules! smoke_dump_test {
        ($name:ident, $code:expr, $slug:expr, [$($pointer:expr),+ $(,)?]) => {
            #[test]
            fn $name() {
                let Some(html) = dump_html($slug) else {
                    return;
                };
                let value = destructure(&html, $code);
                $(assert_string_at(&value, $pointer);)+
            }
        };
    }

    /// The first dumped page for `slug`, or `None` when the dump corpus is absent.
    ///
    /// `data/dumps/offers` is gitignored and populated by `cargo run --bin dev
    /// dump-offer`, so these tests skip rather than fail on a clean checkout.
    fn dump_html(slug: &str) -> Option<String> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/dumps/offers")
            .join(slug);
        let mut entries = fs::read_dir(&dir)
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
            .collect::<Vec<_>>();
        entries.sort();

        fs::read_to_string(entries.into_iter().next()?).ok()
    }

    fn assert_string_at(value: &Value, pointer: &str) {
        let actual = value.pointer(pointer).and_then(Value::as_str);

        assert!(
            actual.is_some_and(|text| !text.trim().is_empty()),
            "missing non-empty string at {pointer} in {value}"
        );
    }

    smoke_dump_test!(
        ankercom_dump_smoke,
        RetailerCode::AnkerCom,
        "ankercom",
        [
            "/schemas/0/sku",
            "/next_data/canonical",
            "/next_data/title",
            "/next_data/price",
            "/product/title",
            "/product/reviews/average_rating",
            "/product/reviews_widget/product_id"
        ]
    );
    smoke_dump_test!(
        ankereu_dump_smoke,
        RetailerCode::AnkerEu,
        "ankereu",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/schemas/0/sku",
            "/schemas/0/price",
            "/head_meta/og_title",
            "/breadcrumbs/0/current",
            "/product/reviews/average_rating"
        ]
    );
    /// Reads a specific dump file by name (not just the first alphabetically,
    /// unlike [`dump_html`]) — for regression coverage of a known edge case.
    ///
    /// Returns `None` when the dump is not in the local corpus: `data/` is
    /// gitignored and each machine holds its own sample, so a named fixture is
    /// not guaranteed to be present. Callers skip rather than fail in that case.
    fn dump_html_named(slug: &str, filename: &str) -> Option<String> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/dumps/offers")
            .join(slug)
            .join(filename);
        match fs::read_to_string(&path) {
            Ok(html) => Some(html),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("skipping: {} is not in the local dump corpus", path.display());
                None
            }
            Err(error) => panic!("failed reading {}: {error}", path.display()),
        }
    }

    /// Regression test for a legacy ankereu template: Key Features / description
    /// content authored via a rich-text editor (`ace-line` divs) rather than
    /// plain `<li>` markup, which on some products the HTML5 parser re-parents
    /// outside `<main>` entirely (see `anker_eu.rs` module docs). Asserts the
    /// top-level `rich_text_blocks` collection actually catches this content.
    #[test]
    fn ankereu_ace_line_legacy_template_smoke() {
        let Some(html) = dump_html_named("ankereu", "eu-en-products-a2331.html") else {
            return;
        };
        let value = destructure(&html, RetailerCode::AnkerEu);
        let blocks = value
            .pointer("/rich_text_blocks")
            .and_then(Value::as_array)
            .filter(|blocks| !blocks.is_empty());
        assert!(
            blocks.is_some(),
            "expected non-empty rich_text_blocks on the ace-line legacy template"
        );
    }

    /// Regression test for a multi-value product option (e.g. multiple colors)
    /// on ankercom — `props.pageProps.product.options[].name` must populate;
    /// the nested `values[].label` path is NOT extracted (the resolver only
    /// supports one `[]` per path — see docs/shopify-extraction-report.md).
    #[test]
    fn ankercom_multi_option_product_smoke() {
        let Some(html) = dump_html_named("ankercom", "products-a1611021-f0.html") else {
            return;
        };
        let value = destructure(&html, RetailerCode::AnkerCom);
        assert_string_at(&value, "/next_data/options/0/option_name");
    }

    /// Documents current, accepted behavior for the two ankereu dumps that
    /// belong to a different brand (Anker SOLIX, not Anker) served under an
    /// anker.com/eu-en URL — see docs/shopify-extraction-report.md. They still
    /// extract without erroring, but under the `AnkerEu` retailer identity,
    /// which is a known, flagged limitation, not a bug this test should hide.
    #[test]
    fn ankereu_solix_brand_dump_smoke() {
        for filename in ["eu-en-products-a17613a1-94.html", "eu-en-products-motion-300.html"] {
            let Some(html) = dump_html_named("ankereu", filename) else {
                continue;
            };
            let value = destructure(&html, RetailerCode::AnkerEu);
            let canonical = value
                .pointer("/next_data/canonical")
                .and_then(Value::as_str)
                .unwrap_or("");
            assert!(
                canonical.contains("ankersolix.com"),
                "{filename}: expected a SOLIX canonical (known cross-brand dump), got {canonical:?}"
            );
        }
    }

    smoke_dump_test!(
        ankernz_dump_smoke,
        RetailerCode::AnkerNz,
        "ankernz",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/schemas/0/sku",
            "/schemas/0/price",
            "/head_meta/og_title",
            "/breadcrumbs/0/current",
            "/nav_main/0/buttons/0/text"
        ]
    );
    smoke_dump_test!(
        ankerpl_dump_smoke,
        RetailerCode::AnkerPl,
        "ankerpl",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/schemas/0/sku",
            "/schemas/0/price",
            "/head_meta/og_title",
            "/breadcrumbs/0/current",
            "/reviews_widget/0/widget/product_id"
        ]
    );
    smoke_dump_test!(
        ankernordicscom_dump_smoke,
        RetailerCode::AnkerNordicsCom,
        "ankernordicscom",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/next_data/shop_host",
            "/schemas/2/name",
            "/schemas/2/price",
            "/head_meta/og_title",
            "/product/title",
            "/product/current_crumb"
        ]
    );
    smoke_dump_test!(
        ugreencom_dump_smoke,
        RetailerCode::UgreenCom,
        "ugreencom",
        [
            "/meta/id",
            "/meta/handle",
            "/meta/vendor",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreende_dump_smoke,
        RetailerCode::UgreenDe,
        "ugreende",
        [
            "/meta/id",
            "/meta/handle",
            "/og_title",
            "/og_price_amount",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreenfr_dump_smoke,
        RetailerCode::UgreenFr,
        "ugreenfr",
        [
            "/meta/id",
            "/meta/handle",
            "/og_title",
            "/og_price",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreennl_dump_smoke,
        RetailerCode::UgreenNl,
        "ugreennl",
        [
            "/meta/id",
            "/meta/handle",
            "/og_title",
            "/og_price_amount",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreenus_dump_smoke,
        RetailerCode::UgreenUs,
        "ugreenus",
        [
            "/meta/id",
            "/meta/handle",
            "/og_title",
            "/og_price_amount",
            "/omega_product_id",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreenca_dump_smoke,
        RetailerCode::UgreenCa,
        "ugreenca",
        [
            "/meta/id",
            "/meta/handle",
            "/schemas/0/name",
            "/product/title",
            "/product/price",
            "/product/breadcrumb_current"
        ]
    );
    smoke_dump_test!(
        ugreenin_dump_smoke,
        RetailerCode::UgreenIn,
        "ugreenin",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreenjp_dump_smoke,
        RetailerCode::UgreenJp,
        "ugreenjp",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreenkr_dump_smoke,
        RetailerCode::UgreenKr,
        "ugreenkr",
        [
            "/meta/id",
            "/meta/handle",
            "/og_title",
            "/schemas/0/name",
            "/product/title"
        ]
    );
    smoke_dump_test!(
        ugreennas_dump_smoke,
        RetailerCode::UgreenNas,
        "ugreennas",
        [
            "/meta/id",
            "/meta/handle",
            "/schemas/0/name",
            "/product/title",
            "/product/price"
        ]
    );
    smoke_dump_test!(
        ugreennasau_dump_smoke,
        RetailerCode::UgreenNasAu,
        "ugreennasau",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasca_dump_smoke,
        RetailerCode::UgreenNasCa,
        "ugreennasca",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasde_dump_smoke,
        RetailerCode::UgreenNasDe,
        "ugreennasde",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennases_dump_smoke,
        RetailerCode::UgreenNasEs,
        "ugreennases",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennaseu_dump_smoke,
        RetailerCode::UgreenNasEu,
        "ugreennaseu",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasfr_dump_smoke,
        RetailerCode::UgreenNasFr,
        "ugreennasfr",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasit_dump_smoke,
        RetailerCode::UgreenNasIt,
        "ugreennasit",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasjp_dump_smoke,
        RetailerCode::UgreenNasJp,
        "ugreennasjp",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ugreennasuk_dump_smoke,
        RetailerCode::UgreenNasUk,
        "ugreennasuk",
        ["/meta/id", "/meta/handle", "/schemas/0/name", "/product/title"]
    );
    smoke_dump_test!(
        ankeritalycom_dump_smoke,
        RetailerCode::AnkerItalyCom,
        "ankeritalycom",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/next_data/handle",
            "/next_data/vendor",
            "/next_data/variants/0/sku",
            "/schemas/2/name",
            "/schemas/2/price",
            "/schemas/3/type",
            "/head_meta/og_title",
            "/head_meta/canonical",
            "/product/title",
            "/product/reviews/average_rating",
            "/product/prices/0/text",
            "/reviews_widget/0/widget/product_id"
        ]
    );
    smoke_dump_test!(
        ankerde_dump_smoke,
        RetailerCode::AnkerDe,
        "ankerde",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/next_data/name",
            "/next_data/variants/0/sku",
            "/schemas/0/name",
            "/schemas/2/type",
            "/head_meta/og_title",
            "/head_meta/canonical",
            "/product/headings/0/text",
            "/product/reviews/average_rating"
        ]
    );
    smoke_dump_test!(
        ankerfr_dump_smoke,
        RetailerCode::AnkerFr,
        "ankerfr",
        [
            "/next_data/title",
            "/next_data/price",
            "/next_data/canonical",
            "/next_data/handle",
            "/next_data/variants/0/sku",
            "/schemas/0/name",
            "/head_meta/og_title",
            "/head_meta/canonical",
            "/product/headings/0/text"
        ]
    );
    smoke_dump_test!(
        minisforumeu_dump_smoke,
        RetailerCode::MinisForumEu,
        "minisforumeu",
        [
            "/locale",
            "/xcotton_pp_variants/title",
            "/xcotton_pp_variants/price",
            "/meta/id",
            "/meta/vendor",
            "/viewed_product/name",
            "/viewed_product/price",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumuk_dump_smoke,
        RetailerCode::MinisForumUk,
        "minisforumuk",
        [
            "/locale",
            "/tt_product/title",
            "/xcotton_pp_variants/title",
            "/xcotton_pp_variants/price",
            "/meta/id",
            "/viewed_product/name",
            "/viewed_product/price",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumau_dump_smoke,
        RetailerCode::MinisForumAu,
        "minisforumau",
        [
            "/locale",
            "/tt_product/title",
            "/product/title",
            "/product/price",
            "/meta/id",
            "/product_variants/0/sku",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumca_dump_smoke,
        RetailerCode::MinisForumCa,
        "minisforumca",
        [
            "/locale",
            "/tt_product/title",
            "/xcotton_pp_variants/title",
            "/xcotton_pp_variants/price",
            "/meta/id",
            "/product_variants/0/sku",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumfr_dump_smoke,
        RetailerCode::MinisForumFr,
        "minisforumfr",
        [
            "/locale",
            "/xcotton_pp_variants/title",
            "/xcotton_pp_variants/price",
            "/meta/id",
            // `/product_variants/0/sku` is deliberately not asserted: gift cards
            // carry `"sku": null`, and one sorts first in this dump folder. The
            // pointer stays covered by the au/ca/jp smoke tests.
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumhk_dump_smoke,
        RetailerCode::MinisForumHk,
        "minisforumhk",
        [
            "/locale",
            "/tt_product/title",
            "/bm_product_variants/0/price",
            "/meta/id",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumjp_dump_smoke,
        RetailerCode::MinisForumJp,
        "minisforumjp",
        [
            "/locale",
            "/product_variants/0/sku",
            "/product_variants/0/price",
            "/meta/id",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumkr_dump_smoke,
        RetailerCode::MinisForumKr,
        "minisforumkr",
        [
            "/locale",
            "/product_variants/0/name",
            "/product_variants/0/price",
            "/meta/id",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumru_dump_smoke,
        RetailerCode::MinisForumRu,
        "minisforumru",
        [
            "/locale",
            "/product_variants/0/title",
            "/product_variants/0/price",
            "/meta/id",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        minisforumus_dump_smoke,
        RetailerCode::MinisForumUs,
        "minisforumus",
        [
            "/locale",
            "/xcotton_pp_variants/title",
            "/xcotton_pp_variants/price",
            "/product/title",
            "/meta/id",
            "/viewed_product/name",
            "/pixels/currency"
        ]
    );
    smoke_dump_test!(
        ugreeneu_dump_smoke,
        RetailerCode::UgreenEu,
        "ugreeneu",
        [
            "/locale",
            "/meta/id",
            "/meta/vendor",
            "/meta/variants/0/sku",
            "/viewed_product/name",
            "/viewed_product/price",
            "/pixels/currency",
            "/product/title",
            "/product/price",
            "/product/compare_at_price"
        ]
    );
    smoke_dump_test!(
        ugreenuk_dump_smoke,
        RetailerCode::UgreenUk,
        "ugreenuk",
        [
            "/locale",
            "/meta/id",
            "/meta/vendor",
            "/meta/variants/0/sku",
            "/viewed_product/name",
            "/viewed_product/price",
            "/pixels/currency",
            "/product/title",
            "/product/price" // `/product/compare_at_price` is deliberately not asserted: the
                             // `.compare-at-price` element only renders on discounted products.
        ]
    );
}
