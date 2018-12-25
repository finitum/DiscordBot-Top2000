import urllib.request
import requests
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


def get_song_by_id(song_id):
    url = "https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid=" + str(song_id)
    with urllib.request.urlopen(url) as url:
        res = json.loads(url.read().decode())
        return res["data"]


def get_current_song_place():
    try:
        on_air = get_now_on_air()["songfile"]
        song_data = get_song_by_id(on_air["songversion"]["id"])
        positions = song_data[0]["positions"]
        place = positions[len(positions) - 1]["position"]
    except KeyError:
        return -1

    return place


def get_img_url(url):
    img = requests.get(url, allow_redirects=False)
    if str(img.status_code).startswith("3"):
        return "https:" + img.headers['Location']
    else:
        return url


if __name__ == "__main__":
    print(get_current_song_place())
