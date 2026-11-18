const languageButton = document.querySelector("[data-language-switch]");
const translatedElements = document.querySelectorAll("[data-tr][data-en]");

function setLanguage(language) {
  document.documentElement.lang = language;

  for (const element of translatedElements) {
    element.textContent = element.getAttribute(`data-${language}`);
  }

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
