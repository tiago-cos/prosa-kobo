import request from 'supertest';
import { MIDDLEWARE_URL } from '../common';

export async function getInitializationResponse(jwt?: string) {
  let req = request(MIDDLEWARE_URL).get('/v1/initialization');

  if (jwt !== undefined) req = req.auth(jwt, { type: 'bearer' });

  return req.send();
}

export async function getTests() {
  let req = request(MIDDLEWARE_URL).post('/v1/analytics/gettests');

  const body = {
    AffiliateName: 'affiliate',
    ApplicationVersion: '2.1.0',
    PlatformId: 'platform_id',
    SerialNumber: '2',
    TestKey: 'test_key'
  };

  return req.send(body);
}

export async function generateInitializationResponse(deviceId: string) {
  const encodedDeviceId = encodeURIComponent(deviceId);
  const url = new URL(MIDDLEWARE_URL);
  let response = INITIALIZATION_RESPONSE_TEMPLATE.replace(/{host}/g, url.host);
  response = response.replace(/{device_id}/g, encodedDeviceId);
  return JSON.parse(response);
}

export async function generateGetTestsResponse() {
  return JSON.parse(GET_TESTS_RESPONSE);
}

const INITIALIZATION_RESPONSE_TEMPLATE = `
{
    "Resources": {
        "productsv2": "http://{host}/v2/products",
        "personalizedrecommendations": "http://{host}/v2/users/personalizedrecommendations",
        "ereaderdevices": "http://{host}/v2/products/EReaderDeviceFeeds",
        "user_wishlist": "http://{host}/v1/user/wishlist",
        "user_platform": "http://{host}/v1/user/platform",
        "user_profile": "http://{host}/v1/user/profile",
        "get_download_link": "http://{host}/v1/library/downloadlink",
        "get_download_keys": "http://{host}/v1/library/downloadkeys",
        "checkout_borrowed_book": "http://{host}/v1/library/borrow",
        "library_sync": "http://{host}/v1/library/sync",
        "library_search": "http://{host}/v1/library/search",
        "library_items": "http://{host}/books",
        "add_entitlement": "http://{host}/{RevisionIds}",
        "delete_entitlement": "http://{host}/v1/library/{Ids}",
        "tags": "http://{host}/v1/library/tags",
        "autocomplete": "http://{host}/v1/products/autocomplete",
        "user_reviews": "http://{host}/v1/user/reviews",
        "user_ratings": "http://{host}/v1/user/ratings",
        "user_recommendations": "http://{host}/v1/user/recommendations",
        "taste_profile": "http://{host}/v1/products/tasteprofile",
        "fte_feedback": "http://{host}/v1/products/ftefeedback",
        "shelfie_recommendations": "http://{host}/v1/user/recommendations/shelfie",
        "featured_lists": "http://{host}/v1/products/featured",
        "daily_deal": "http://{host}/v1/products/dailydeal",
        "category": "http://{host}/v1/categories/{CategoryId}",
        "browse_history": "http://{host}/v1/user/browsehistory",
        "notifications_registration_issue": "http://{host}/v1/notifications/registration",
        "exchange_auth": "http://{host}/v1/auth/exchange",
        "rakuten_token_exchange": "http://{host}/v1/auth/rakuten_token_exchange",
        "device_auth": "http://{host}/v1/auth/device",
        "device_refresh": "http://{host}/v1/auth/refresh",
        "add_device": "http://{host}/v1/user/add-device",
        "get_tests_request": "http://{host}/v1/analytics/gettests",
        "post_analytics_event": "http://{host}/v1/analytics/event",
        "user_loyalty_benefits": "http://{host}/v1/user/loyalty/benefits",
        "reading_state": "http://{host}/v1/library/{Ids}/state",
        "library_metadata": "http://{host}/books/{Ids}/metadata",
        "update_accessibility_to_preview": "http://{host}/v1/library/{EntitlementIds}/preview",
        "rename_tag": "http://{host}/v1/library/tags/{TagId}",
        "delete_tag": "http://{host}/v1/library/tags/{TagId}",
        "quickbuy_create": "http://{host}/v1/store/quickbuy/purchase",
        "audiobook_purchase_withcredit": "http://{host}/v1/store/audiobook/{Id}",
        "product_reviews": "http://{host}/v1/products/{ProductIds}/reviews",
        "review": "http://{host}/v1/products/reviews/{ReviewId}",
        "product_recommendations": "http://{host}/v1/products/{ProductId}/recommendations",
        "product_nextread": "http://{host}/v1/products/{ProductIds}/nextread",
        "product_prices": "http://{host}/v1/products/{ProductIds}/prices",
        "book": "http://{host}/v1/products/books/{ProductId}",
        "audiobook": "http://{host}/v1/products/audiobooks/{ProductId}",
        "book_subscription": "http://{host}/v1/products/books/subscriptions",
        "related_items": "http://{host}/v1/products/{Id}/related",
        "featured_list": "http://{host}/v1/products/featured/{FeaturedListId}",
        "category_featured_lists": "http://{host}/v1/categories/{CategoryId}/featured",
        "category_products": "http://{host}/v1/categories/{CategoryId}/products",
        "library_prices": "http://{host}/v1/user/library/previews/prices",
        "library_book": "http://{host}/books/{LibraryItemId}",
        "tag_items": "http://{host}/v1/library/tags/{TagId}/Items",
        "quickbuy_checkout": "http://{host}/v1/store/quickbuy/{PurchaseId}/checkout",
        "rating": "http://{host}/v1/products/{ProductId}/rating/{Rating}",
        "authorproduct_recommendations": "http://{host}/v1/products/books/authors/recommendations",
        "external_book": "http://{host}/v1/products/books/external/{Ids}",
        "remaining_book_series": "http://{host}/v1/products/books/series/{SeriesId}",
        "audiobook_preview": "http://{host}/v1/products/audiobooks/{Id}/preview",
        "content_access_book": "http://{host}/v1/products/books/{ProductId}/access",
        "delete_tag_items": "http://{host}/v1/library/tags/{TagId}/items/delete",
        "review_sentiment": "http://{host}/v1/products/reviews/{ReviewId}/sentiment/{Sentiment}",
        "products": "http://{host}/v1/products",
        "categories": "http://{host}/v1/categories",
        "funnel_metrics": "http://{host}/v1/funnelmetrics",
        "deals": "http://{host}/v1/deals",
        "configuration_data": "http://{host}/v1/configuration",
        "assets": "http://{host}/v1/assets",
        "affiliaterequest": "http://{host}/v1/affiliate",
        "notebooks": "http://{host}/api/internal/notebooks",
        "image_host": "http://{host}/images/",
        "store_host": "www.kobo.com",
        "store_home": "www.kobo.com/{region}/{language}",
        "social_authorization_host": "https://social.kobobooks.com:8443",
        "social_host": "https://social.kobobooks.com",
        "reading_services_host": "http://{host}",
        "discovery_host": "https://discovery.kobobooks.com",
        "oauth_host": "http://{host}/oauth/{device_id}",
        "eula_page": "https://www.kobo.com/termsofuse?style=onestore",
        "password_retrieval_page": "https://www.kobo.com/passwordretrieval.html",
        "store_search": "https://www.kobo.com/{region}/{language}/Search?Query={query}",
        "store_top50": "https://www.kobo.com/{region}/{language}/ebooks/Top",
        "store_newreleases": "https://www.kobo.com/{region}/{language}/List/new-releases/961XUjtsU0qxkFItWOutGA",
        "privacy_page": "https://www.kobo.com/privacypolicy?style=onestore",
        "terms_of_sale_page": "https://authorize.kobo.com/{region}/{language}/terms/termsofsale",
        "book_detail_page": "https://www.kobo.com/{region}/{language}/ebook/{slug}",
        "book_detail_page_rakuten": "http://books.rakuten.co.jp/rk/{crossrevisionid}",
        "book_landing_page": "https://www.kobo.com/ebooks",
        "magazine_landing_page": "https://www.kobo.com/emagazines",
        "purchase_buy": "https://www.kobo.com/checkoutoption/",
        "purchase_buy_templated": "https://www.kobo.com/{region}/{language}/checkoutoption/{ProductId}",
        "love_points_redemption_page": "https://www.kobo.com/{region}/{language}/KoboSuperPointsRedemption?productId={ProductId}",
        "categories_page": "https://www.kobo.com/ebooks/categories",
        "redeem_interstitial_page": "https://www.kobo.com",
        "love_dashboard_page": "https://www.kobo.com/{region}/{language}/kobosuperpoints",
        "help_page": "http://www.kobo.com/help",
        "image_url_template": "http://{host}/images/{ImageId}&width={Width}&height={Height}",
        "image_url_quality_template": "http://{host}/images/{ImageId}&width={Width}&height={Height}",
        "customer_care_live_chat": "https://v2.zopim.com/widget/livechat.html?key=Y6gwUmnu4OATxN3Tli4Av9bYN319BTdO",
        "audiobook_landing_page": "https://www.kobo.com/{region}/{language}/audiobooks",
        "userguide_host": "https://ereaderfiles.kobo.com",
        "dictionary_host": "https://ereaderfiles.kobo.com",
        "audiobook_detail_page": "https://www.kobo.com/{region}/{language}/audiobook/{slug}",
        "wishlist_page": "https://www.kobo.com/{region}/{language}/account/wishlist",
        "audiobook_subscription_orange_deal_inclusion_url": "https://authorize.kobo.com/inclusion",
        "giftcard_redeem_url": "https://www.kobo.com/{storefront}/{language}/redeem",
        "giftcard_epd_redeem_url": "https://www.kobo.com/{storefront}/{language}/redeem-ereader",
        "account_page": "https://www.kobo.com/account/settings",
        "account_page_rakuten": "https://my.rakuten.co.jp/",
        "pocket_link_account_start": "https://authorize.kobo.com/{region}/{language}/linkpocket",
        "client_authd_referral": "https://authorize.kobo.com/api/AuthenticatedReferral/client/v1/getLink",
        "dropbox_link_account_start": "https://authorize.kobo.com/LinkDropbox/start",
        "dropbox_link_account_poll": "https://authorize.kobo.com/{region}/{language}/LinkDropbox",
        "subs_management_page": "https://www.kobo.com/{region}/{language}/account/subscriptions",
        "subs_landing_page": "https://www.kobo.com/{region}/{language}/plus",
        "subs_purchase_buy_templated": "https://www.kobo.com/{region}/{language}/Checkoutoption/{ProductId}/{TierId}",
        "subs_plans_page": "https://www.kobo.com/{region}/{language}/plus/plans",
        "sign_in_page": "https://auth.kobobooks.com/ActivateOnWeb",
        "more_sign_in_options": "https://authorize.kobo.com/signin?returnUrl=http://kobo.com/#allProviders",
        "registration_page": "https://authorize.kobo.com/signup?returnUrl=http://kobo.com/",
        "facebook_sso_page": "https://authorize.kobo.com/signin/provider/Facebook/login?returnUrl=http://kobo.com/",
        "provider_external_sign_in_page": "https://authorize.kobo.com/ExternalSignIn/{providerName}?returnUrl=http://kobo.com/",
        "free_books_page": {
        "EN": "https://www.kobo.com/{region}/{language}/p/free-ebooks",
        "FR": "https://www.kobo.com/{region}/{language}/p/livres-gratuits",
        "IT": "https://www.kobo.com/{region}/{language}/p/libri-gratuiti",
        "NL": "https://www.kobo.com/{region}/{language}/List/bekijk-het-overzicht-van-gratis-ebooks/QpkkVWnUw8sxmgjSlCbJRg",
        "PT": "https://www.kobo.com/{region}/{language}/p/livros-gratis"
        },
        "blackstone_header": {
            "key": "x-amz-request-payer",
            "value": "requester"
        },
        "use_one_store": "True",
        "kobo_superpoints_enabled": "False",
        "kobo_subscriptions_enabled": "True",
        "kobo_onestorelibrary_enabled": "False",
        "kobo_nativeborrow_enabled": "False",
        "kobo_audiobooks_enabled": "True",
        "kobo_audiobooks_subscriptions_enabled": "False",
        "kobo_audiobooks_credit_redemption": "False",
        "kobo_audiobooks_orange_deal_enabled": "False",
        "kobo_wishlist_enabled": "True",
        "kobo_shelfie_enabled": "False",
        "kobo_redeem_enabled": "True",
        "kobo_dropbox_link_account_enabled": "True",
        "kobo_display_price": "True",
        "kobo_google_tax": "False",
        "kobo_googledrive_link_account_enabled": "False",
        "kobo_onedrive_link_account_enabled": "False",
        "kobo_privacyCentre_url": "https://www.kobo.com/privacy",
        "gpb_flow_enabled": "False",
        "ppx_purchasing_url": "https://purchasing.kobo.com"
    }
}`;

const GET_TESTS_RESPONSE = `
{
    "Result": "Success",
    "TestKey": "test_key",
    "Tests": {}
}`;
