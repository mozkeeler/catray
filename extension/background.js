const kTotalCats = 1363;

function listener(info) {
  try {
    if (info.url.startsWith("https://pbs.twimg.com/profile_images/")) {
      // https://pbs.twimg.com/profile_images/<id number?>/<some other identifier?>_<size>.png
      let id = info.url.split("/")[4];
      let catId = parseInt(id) % kTotalCats;
      let catIdStr = catId.toString().padStart(8, "0");
      return { redirectUrl: browser.extension.getURL("cats/" + catIdStr + "-96.jpg") };
    }
  } catch (e) {
    console.error(e);
  }

  return {};
}

browser.webRequest.onBeforeRequest.addListener(
  listener,
  { urls: ["https://pbs.twimg.com/*"], types: ["image"] },
  ["blocking"]
);
