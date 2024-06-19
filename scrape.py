import json
from typing import TypeVar, cast

import requests
from bs4 import BeautifulSoup
from rich import inspect

T = TypeVar("T")


def notnull(x: T | None) -> T:
    return cast(T, x)


urls = [
    "https://products.basf.com/global/en/ci/n-vinyl-2-pyrrolidone.html",
    "https://pubchem.ncbi.nlm.nih.gov/compound/N-Vinyl-2-pyrrolidone",
    "https://www.shokubai.co.jp/en/products/detail/nvp/",
    "https://pubchem.ncbi.nlm.nih.gov/compound/N-Vinyl-2-pyrrolidone",
    "https://www.sciencedirect.com/topics/pharmacology-toxicology-and-pharmaceutical-science/1-vinyl-2-pyrrolidinone",
    "https://www.ncbi.nlm.nih.gov/books/NBK498761/#:~:text=It%20is%20used%20in%20the,the%20synthesis%20of%20phenolic%20resins",
    "https://www.sciencedirect.com/topics/agricultural-and-biological-sciences/polyvinylpyrrolidone#:~:text=PVP%20added%20to%20iodine%20forms,trade%20name%20Betadine%20and%20Pyodine",
    "https://www.shokubai.co.jp/en/products/detail/nvp/#:~:text=N%2Dvinylpyrrolidone%20is%20a%20nonionic,monomer%20with%20the%20following%20features.&text=N%2Dvinylpyrrolidone%20is%20used%20as,of%20reactivity%20with%20UV%20irradiation",
    "https://adhesives.specialchem.com/product/m-basf-n-vinyl-pyrrolidone-nvp",
    "https://www.welinkschem.com/nvp-n-vinyl-pyrrolidone/",
    "https://pubs.rsc.org/en/content/articlelanding/2019/py/c8py01459k",
    "https://www.science.gov/topicpages/n/n-vinyl+pyrrolidone+nvp",
    "https://shdexiang.en.made-in-china.com/product/tXfQDioPsKVn/China-N-Vinylpyrrolidone-CAS-No-88-12-0-C6h9no.html",
    "https://www.cphi-online.com/nvp-n-vinylpyrrolidone-prod1288298.html",
    "https://www.mdpi.com/2073-4360/11/6/1079",
]


def scrape(url: str):
    response = requests.get(
        url,
        headers={
            "User-Agent": "Mozilla/5.0 (X11; Linux x86_64; rv:125.0) Gecko/20100101 Firefox/125.0"
        },
    )
    if not response.ok:
        inspect(response)
        msg = f"RESPONSE NOT OK: `{url}`"
        raise ValueError(msg)
    soup = BeautifulSoup(response.text, "lxml")
    return {
        "title": notnull(soup.title).string,
        "images": [*filter(bool, (img.get("src") for img in soup.find_all("img")))],
        "links": [*filter(bool, (a.get("href") for a in soup.find_all("a")))],
    }


with open("data.json", "w") as f:
    json.dump({url: scrape(url) for url in urls}, f)
