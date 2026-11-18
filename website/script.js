const languageButton = document.querySelector("[data-language-switch]");
const translatedElements = document.querySelectorAll("[data-tr][data-en]");
const accessibleLabels = document.querySelectorAll(
  "[data-tr-aria-label][data-en-aria-label]",
);
const pageTitle = document.querySelector("#page-title");
const pageDescription = document.querySelector("#page-description");

function setLanguage(language) {
  document.documentElement.lang = language;

  for (const element of translatedElements) {
    element.textContent = element.getAttribute(`data-${language}`);
  }

  for (const element of accessibleLabels) {
    element.setAttribute("aria-label", element.getAttribute(`data-${language}-aria-label`));
  }

  document.title = pageTitle.getAttribute(`data-${language}`);
  pageDescription.setAttribute("content", pageDescription.getAttribute(`data-${language}-content`));
  languageButton.textContent = language === "tr" ? "EN" : "TR";
  languageButton.setAttribute(
    "aria-label",
    language === "tr" ? "Switch to English" : "Türkçeye geç",
  );
}

setLanguage("tr");

languageButton.addEventListener("click", () => {
  setLanguage(document.documentElement.lang === "tr" ? "en" : "tr");
});
