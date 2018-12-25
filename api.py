import urllib.request
import json


def get_whole_list():
    url = "https://www.nporadio2.nl/?option=com_ajax&plugin=Top2000&format=json&year=2018"
    with urllib.request.urlopen(url) as url:
        res = json.loads(url.read().decode())
        return res["data"]


def get_now_on_air():
    with urllib.request.urlopen("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json") as url:
        res = json.loads(url.read().decode())
        return res['results'][0]


def get_current_song():
    data = get_now_on_air()
    song = data["songfile"]["title"] + " by " + data["songfile"]["artist"]
    return song


if __name__ == "__main__":
    print(get_whole_list())


