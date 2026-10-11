# Brief

Stage 1. `C1 Frame the problem and set the rules`, `B1 Establish context`.

## The request, word for word

Sam passed on a friend's prompt that holds the game's design decisions. It arrived pasted twice in one message; it is quoted once here, unchanged.

> Make me a game about lowballing sellers on an online sales platform similar to facebook marketplace. The player is presented with a grid of six listings for various items. The player can click on each item to see more details. Each item has a photo and a description of what is being sold, as well as how long the item has been listed for in hours or days. The description may or may not include phrases such as "offers accepted", "or best offer", "price is firm", etc.. The listing also has a button "Inspect Profile" and another button "Message Seller".
>
> Every item has a minimum price that the seller will accept, but it is not known to the player. The seller's minimum price should randomly vary but also correspond somewhat strongly with the tone of the listing. For example, a listing which uses the phrase "firm" would have a seller's minimum price closer to the list price than a listing that says "offers accepted".
>
> For each screen of six items, the player may inspect the profile of up to two item sellers and send a message to up to two item sellers. The player may not use both the "Inspect Profile" and "Message Seller" button on the same listing.
> Each seller has a tendency to accept lowball offers or not. This is inherent to each seller and is consistently reflected in the way that they write their descriptions, what can be learned by inspecting their profile, and the way that they write their messages.
>
> For each grid of six listings, the player must make an offer on five items. They can skip one item. The player scores points in the form of dollars saved by lowballing offers. So, if a player offers $80 on a $100 item and the offer is accepted, the player scores $100 minus $80, or $20 in points. If the player's offer is lower than the minimum price that the seller will accept, the player scores $0 in points and the item is still counted as one of the five selections. The player's total points are added up for the round and the minimum seller price for the sixth, unbid item is shown.
> The items for sale should be various common items represented in a 2D drawing in Corporate Memphis or Alegria art style. The list price should correspond to real-life reasonable prices, for example a toaster for $20, signed football jersey for $300, or a 15 year old used car for $4000. The seller's minimum acceptable price should be a function of the item's price, the seller's description, and the seller's attitude, plus some degree of randomness.Often, list prices should be in multiples of 5 or 10 dollars for lower priced items under $100, and in increments of $50-$100 for items higher in price. The seller's minimum selling price can be more specific, such as $77 for a $100 item.
>
> The player goes through three screens of six items, and then their score is tallied at the end as a percentage of money spent, i.e. they on average lowballed 23%.

Sam's own instructions, sent with it:

> Please use the Builder repo as a base in the directory above, and use the work and repos of Sam gilson at sgilson7.github.io, which can all be found one directory up. please build out the MVP and deploy it to my website. you have approval for creating a public github repo, github pages, and approval to deploy to my website when its ready. any time you need to make 2D art and any kind of asset for the game, use the tikz latex prompt you can find in other repos, and use sam comm final as an inspiration image, alongside the menu screens of all of the sgilson7 games which use a similar build pipeline for images. now build the mvp for the game and deploy it, dont stop till its deployed, good luck. here is the prompt from my friend to follow to create the game containing all design decisions: the game should be 2d

## The game

Lowballer: a 2D browser game on a made-up marketplace, Curbside. Three screens of six listings; on each the player makes five offers, may skip one, may peek at two seller profiles and message two sellers (never both on one listing). Played with a mouse or a finger, on a laptop or a phone.

## Done means

- Three screens of six listings, each with a TikZ picture, a price, a title, how long it has been listed, a description, Inspect Profile and Message Seller.
- Every seller has a hidden flexibility. It drives the tone phrase in the description, the voice of the description, the profile (bio, reviews, response time, recent sales) and the reply to a message, and, with the tone, the listing's age and some noise, the hidden minimum.
- Two profile peeks and two messages per screen, never both on one listing; five offers per screen.
- An accepted offer scores asking minus offer; a declined one scores $0 and still counts. The round's total is shown with the skipped item's minimum.
- After three screens: total saved, and saved as a percentage of the asking prices of the items bid on.
- `./scripts/check.sh` passes; the game is live on GitHub Pages and listed on sgilson7.github.io.

## Decisions that stay with the person

- The game's name and the platform's name (the agent chose Lowballer and Curbside; change in `data/copy.json`).
- The tuning of the minimum-price formula (`crates/core/src/market.rs`).

## Autonomy

Sam approved in advance: a public GitHub repository, GitHub Pages for it, and listing and deploying it on sgilson7.github.io when ready. The plan is approved in advance ("build the MVP and deploy it, don't stop till it's deployed"). A push to `main` deploys after the check passes.

## Allowed core dependencies

`serde`, `serde_json` (reference default).
