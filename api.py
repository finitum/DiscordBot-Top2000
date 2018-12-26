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
        if curr_record["s"] == title and curr_record["a"] == artist:
            return curr_record


def get_song_by_id(song_id):
    url = "https://www.nporadio2.nl/?option=com_ajax&plugin=Trackdata&format=json&songid=" + str(song_id)
    with urllib.request.urlopen(url) as url:
        res = json.loads(url.read().decode())
        return res["data"][0]


def get_now_on_air():
    with urllib.request.urlopen("https://radiobox2.omroep.nl/data/radiobox2/nowonair/2.json") as url:
        res = json.loads(url.read().decode())
        return res['results'][0]


def get_now_on_air_id():
    on_air = get_now_on_air()["songfile"]
    try:
        on_air_id = on_air["songversion"]["id"]
        return on_air_id
    except KeyError:
        title = on_air["title"]
        artist = on_air["artist"]

        search = search_song_by_name(title, artist)
        return search["aid"]


def get_now_on_air_details():
    song = get_song_by_id(get_now_on_air_id())
    return song


def get_now_on_air_from_full_list():
    on_air = get_now_on_air()["songfile"]

    title = on_air["title"]
    artist = on_air["artist"]

    search = search_song_by_name(title, artist)
    return search


def get_current_song_place():
    return get_now_on_air_from_full_list()["pos"]


def get_img_url(url):
    img = requests.get(url, allow_redirects=False)
    if str(img.status_code).startswith("3"):
        return "https:" + img.headers['Location']
    else:
        return url


if __name__ == "__main__":
    print(get_current_song_place())
