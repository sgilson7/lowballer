"""Lowballer's browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. This plays a whole game the way a person would and asserts what the
page shows. Expected values are worked out by hand from the brief (C4), never
copied from the game's own output:

- every offer is $1, which is below every seller's minimum (the deepest
  discount is 45%, and the cheapest asking price is $15), so all fifteen are
  declined, each screen saves $0, and the final tally is 0.0%, the lowest rank;
- the sixth listing on each screen is skipped, so its row says "Skipped".
"""

import json
import pathlib

COPY = json.loads((pathlib.Path(__file__).resolve().parents[2] / "data" / "copy.json").read_text())


def open_card(page, k):
    page.locator("#grid .card").nth(k).click()
    page.wait_for_selector("#detail[open]")


def close_sheet(page):
    page.locator("#detail .close").click()
    page.wait_for_selector("#detail:not([open])", state="attached")


def offer(page, k, amount):
    open_card(page, k)
    page.fill("#offer-input", amount)
    page.locator("#detail form.offer button[type=submit]").click()


def run(page, check, engine):
    check(page.inner_text("#title-name") == COPY["game_name"], "the title screen names the game")
    check(page.locator("#title-how li").count() == len(COPY["how_to_play"]), "the title screen explains the rules")
    page.click("#start")
    check(page.locator("#grid .card").count() == 6, "a screen shows six listings")
    check(page.inner_text("#screen") == "Screen 1 of 3", "the top bar says screen 1 of 3")

    # Inspect a profile, then the same listing cannot be messaged.
    open_card(page, 0)
    check(page.locator("#detail .sheet-info .desc").inner_text() != "", "a listing has a description")
    page.locator("#detail .actions button").nth(0).click()
    check(page.locator("#detail .profile").count() == 1, "Inspect Profile shows the seller's profile")
    check(page.locator("#detail .actions button").nth(1).is_disabled(), "Message Seller is off on a listing whose profile was inspected")
    check(COPY["errors"]["used-peek-here"] in page.inner_text("#detail"), "the sheet says why Message Seller is off")
    close_sheet(page)

    # Message a seller: the reply appears.
    open_card(page, 1)
    page.locator("#detail .actions button").nth(1).click()
    check(page.locator("#detail .bubble.theirs").count() == 1, "Message Seller shows the seller's reply")
    check(COPY["question"] in page.inner_text("#detail .bubble.mine"), "the player's question is shown")
    close_sheet(page)

    # The second peek uses the last one; a third listing cannot be inspected.
    open_card(page, 2)
    page.locator("#detail .actions button").nth(0).click()
    close_sheet(page)
    open_card(page, 3)
    check(page.locator("#detail .actions button").nth(0).is_disabled(), "a third profile peek is refused")
    check(COPY["errors"]["no-peeks-left"] in page.inner_text("#detail"), "the sheet says both peeks are used")

    # An offer above asking is refused with the reason.
    page.fill("#offer-input", "99999")
    check(page.inner_text("#offer-hint") == COPY["errors"]["over-asking"], "the hint flags an offer over asking")
    page.locator("#detail form.offer button[type=submit]").click()
    check(page.inner_text("#sheet-error") == COPY["errors"]["over-asking"], "an offer over asking is refused")
    close_sheet(page)

    for screen in (1, 2, 3):
        for k in range(5):
            offer(page, k, "1")
        page.wait_for_selector("#summary:not([hidden])")
        check(page.locator("#rows li").count() == 6, f"screen {screen}'s results list all six listings")
        check(page.locator("#rows .pill-declined").count() == 5, f"screen {screen}: five $1 offers are declined")
        check(page.locator("#rows .pill-skipped").count() == 1, f"screen {screen}: the unbid listing is skipped")
        check("$" in page.locator("#rows .row-skipped p").inner_text(), f"screen {screen}: the skipped listing's minimum is shown")
        check(page.inner_text("#total") == "$0", f"screen {screen} saves $0")
        page.click("#next")
        if screen < 3:
            page.wait_for_selector("#browse:not([hidden])")
            check(page.inner_text("#screen") == f"Screen {screen + 1} of 3", f"the game moves to screen {screen + 1}")

    page.wait_for_selector("#final:not([hidden])")
    check(page.inner_text("#final-pct") == "0.0%", "the final tally is 0.0% when nothing was accepted")
    check(page.inner_text("#final-rank") == COPY["ranks"][0]["name"], "the lowest rank is given")
    page.click("#again")
    page.wait_for_selector("#browse:not([hidden])")
    check(page.inner_text("#screen") == "Screen 1 of 3", "Play again starts a new game at screen 1")
