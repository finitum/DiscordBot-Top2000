import urllib.request
import requests
import json


def get_whole_list():
    url = "https://www.nporadio2.nl/?option=com_ajax&plugin=Top2000&format=json&year=2018"
    with urllib.request.urlopen(url) as url:
        res = json.loads(url.read().decode())
        return res["data"][0]


def search_song_by_name(title, artist):
    whole_list = get_whole_list()
    for curr_record in whole_list:
        if curr_record["s"].lower() == title.lower() and curr_record["a"].lower() == artist.lower():
            return curr_record
    return {
        "aid": -1,
        "url": "/",
        "pos": "(error)",
        "prv": 0
    }


def get_song_by_id(song_id):
    if song_id == -1:
        return {
            "description": "(error)"
        }

    url = "https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid=" + str(song_id)
    with urllib.request.urlopen(url) as url:
        res = json.loads(url.read().decode())
        return res["data"][0]


def get_now_on_air():
    with urllib.request.urlopen("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json") as url:
        res = json.loads(url.read().decode())
        return res['results'][0]


def get_now_on_air_id(on_air):
    try:
        on_air_id = on_air["songversion"]["id"]
        return on_air_id
    except KeyError:
        title = on_air["title"]
        artist = on_air["artist"]

        search = search_song_by_name(title, artist)
        if search["aid"] != -1:
            return search["aid"]
        else:
            return -1


def get_now_on_air_details(on_air):
    song = get_song_by_id(get_now_on_air_id(on_air))
    return song


def get_now_on_air_from_full_list(on_air):

    title = on_air["title"]
    artist = on_air["artist"]

    search = search_song_by_name(title, artist)
    return search


def get_img_url(url):
    img = requests.get(url, allow_redirects=False)
    if str(img.status_code).startswith("3"):
        return "https:" + img.headers['Location']
    else:
        return url
