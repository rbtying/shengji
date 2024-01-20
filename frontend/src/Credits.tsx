import * as React from "react";
import * as ReactModal from "react-modal";
import { AppStateContext } from "./AppStateProvider";

const contentStyle: React.CSSProperties = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
};

const changeLogVersion: number = 23;

const ChangeLog = (): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
  const { state, updateState } = React.useContext(AppStateContext);
  React.useEffect(() => {
    if (state.changeLogLastViewed !== changeLogVersion) {
      setModalOpen(true);
    }
  }, []);
  return (
    <>
      <a
        onClick={(evt) => {
          evt.preventDefault();
          setModalOpen(true);
        }}
        href={window.location.href}
      >
        Change Log
      </a>
      <ReactModal
        isOpen={modalOpen}
        onRequestClose={() => {
          setModalOpen(false);
          updateState({ changeLogLastViewed: changeLogVersion });
        }}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{ content: contentStyle }}
      >
        <h2>For new players</h2>
        <p>
          If you haven&apos;t learned to play the game yet, consider reading the{" "}
          <a href="rules.html" target="_blank">
            rules
          </a>
          .
        </p>
        <p>
          There are a wide variety of game settings which may suit the way you
          normally play, e.g. changing how many decks, how scoring works, etc.
          These can be changed before every round.
        </p>
        <p>
          There are also a bunch of UI customizations that you may want to turn
          on (or leave off) -- click the gear icon at the top of the screen once
          you&apos;re in the game.
        </p>
        <h2>Change Log</h2>
        <p>7/10/2023:</p>
        <ul>
          <li>
            Added a confirmation check from another player when resetting the
            game
          </li>
        </ul>
        <p>1/20/2024:</p>
        <ul>
          <li>
            Added the ability to protect both longer tuples and tractors at the
            same time.
          </li>
        </ul>
        <p>2/24/2023:</p>
        <ul>
          <li>
            Added the ability to list a room publicly for others to join (thanks
            jimmyfang94!)
          </li>
        </ul>
        <p>1/18/2023:</p>
        <ul>
          <li>Fixed performance issue when playing tricks with many cards</li>
          <li>Added suggested play button</li>
        </ul>
        <p>1/11/2023:</p>
        <ul>
          <li>Fixed rendering of card icons</li>
        </ul>
        <p>1/10/2023:</p>
        <ul>
          <li>
            Changed default UI setting to &ldquo;show cards in player
            order&rdquo;.
          </li>
          <li>
            Added icons (can be turned off) for point cards and trump cards.
          </li>
        </ul>
        <p>12/28/2023:</p>
        <ul>
          <li>
            Fix bug where over-trumping a trumped throw would sometimes not
            work.
          </li>
        </ul>
        <p>9/25/2022:</p>
        <ul>
          <li>Add the ability to disable joker bids</li>
          <li>Add the ability to set autodraw speed</li>
        </ul>
        <p>4/17/2022:</p>
        <ul>
          <li>
            Add the ability to hide the chat box and move the points progress
            bar in the settings pane.
          </li>
        </ul>
        <p>3/07/2022:</p>
        <ul>
          <li>
            Add the ability to choose the final level (NT or A) in difficulty
            settings.
          </li>
        </ul>
        <p>2/05/2022:</p>
        <ul>
          <li>Added long-missing support for no-trump rank after A.</li>
          <li>Added setting to show player name in title bar.</li>
          <li>Added ability to set meta-rank (behind checkbox).</li>
        </ul>
        <p>7/04/2021:</p>
        <ul>
          <li>
            Added option to customize the minimum tractor size under &ldquo;more
            game settings&rdquo;
          </li>
        </ul>
        <p>6/18/2021:</p>
        <ul>
          <li>
            Added option for higher suit non-joker bids to outbid non-joker
            bids.
          </li>
        </ul>
        <p>3/21/2021:</p>
        <ul>
          <li>Added option to view (most of) the UI in dark mode.</li>
          <li>Added button to randomize the player order.</li>
          <li>Added button to check if everyone is ready.</li>
        </ul>
        <p>3/15/2021:</p>
        <ul>
          <li>
            Added option in Finding Friends to select friends using trumps.
          </li>
        </ul>
        <p>2/15/2021:</p>
        <ul>
          <li>Support protecting tractors from four-of-a-kind.</li>
        </ul>
        <p>2/4/2021:</p>
        <ul>
          <li>
            Support configuring custom deck properties, like short decks or
            removing jokers.
          </li>
        </ul>
        <p>2/2/2021:</p>
        <ul>
          <li>
            Fix bug where unselecting cards would temporarily remove them from
            the game.
          </li>
        </ul>
        <p>1/31/2021:</p>
        <ul>
          <li>
            Players can now choose kitty sizes which require cards to be removed
            from the game.
          </li>
        </ul>
        <p>1/27/2021:</p>
        <ul>
          <li>
            When ending the game early, let players see what cards were
            remaining.
          </li>
        </ul>
        <p>1/22/2021:</p>
        <ul>
          <li>
            Add the ability to end the game early when there are insufficient
            points remaining to matter.
          </li>
        </ul>
        <p>1/21/2021:</p>
        <ul>
          <li>Add a scoring progress bar with point thresholds.</li>
          <li>
            Add a setting to prevent friends from joining twice (in difficulty
            settings).
          </li>
        </ul>
        <p>1/18/2021:</p>
        <ul>
          <li>
            Amend the &ldquo;PointCardNotAllowed&rdquo; friend selection policy.
            King is now a valid friend when the landlord&apos;s rank is Ace.
          </li>
        </ul>
        <p>1/8/2021:</p>
        <ul>
          <li>
            Add settings for reinforcing a bid after it has been overturned, and
            for overbidding yourself.
          </li>
          <li>
            Add a setting to show debug information, to help with more detailed
            bug reports.
          </li>
        </ul>
        <p>12/11/2020:</p>
        <ul>
          <li>Support a friend selection policy that disallows point cards.</li>
        </ul>
        <p>12/07/2020:</p>
        <ul>
          <li>
            Add a setting which hides the indication of which player that can
            defeat a throw.
          </li>
          <li>
            Add a card-protection setting which disables format-based play
            requirements.
          </li>
        </ul>
        <p>11/22/2020:</p>
        <ul>
          <li>
            More rigorously define trick-format decomposition, especially when
            more than four decks are involved. See the issues referenced in{" "}
            <a
              href="https://github.com/rbtying/shengji/pull/258/files"
              target="_blank"
              rel="noreferrer"
            >
              PR #258
            </a>{" "}
            for details.
          </li>
        </ul>
        <p>11/13/2020:</p>
        <ul>
          <li>
            Fix bug in longest-component throw-evaluation policy where the
            winner for tricks of single cards was always the first player.
          </li>
        </ul>
        <p>11/11/2020:</p>
        <ul>
          <li>
            Support a throw evaluation policy based on the longest component.
          </li>
        </ul>
        <p>11/01/2020:</p>
        <ul>
          <li>
            Support more granular step sizes in scoring settings on a
            per-number-of-decks basis.
          </li>
        </ul>
        <p>9/27/2020:</p>
        <ul>
          <li>
            Support limiting joker/no-trump bids in games with more than two
            decks.
          </li>
        </ul>
        <p>9/18/2020:</p>
        <ul>
          <li>Fix performance issues in long games.</li>
        </ul>
        <p>8/30/2020:</p>
        <ul>
          <li>Support end of game kitty reveal.</li>
        </ul>
        <p>8/09/2020:</p>
        <ul>
          <li>Support configuring different score thresholds for each game.</li>
        </ul>
        <p>8/07/2020:</p>
        <ul>
          <li>Allow card colors to be customized</li>
          <li>
            Add option to play sound during draw. Sound sourced from
            dersuperanton at freesound.org
          </li>
        </ul>
        <p>8/02/2020:</p>
        <ul>
          <li>Support beeps in exchange phase</li>
        </ul>
        <p>7/26/2020:</p>
        <ul>
          <li>
            Fix a bug where throws in trump of the trump-rank-card would be
            incorrectly blocked
          </li>
          <li>
            Implement helper which lets you know what plays you can make and
            tells you about format-decompositions
          </li>
          <li>
            Allow player to specify preferred grouping in case of ambiguity,
            e.g. 22333 as either [22][333] or [2233][3]
          </li>
          <li>Add UI hint which shows you cards in the same suit</li>
          <li>Add UI setting which allows you to separate cards by suit</li>
        </ul>
        <p>7/23/2020:</p>
        <ul>
          <li>
            Move a bunch of settings into modals to make interface cleaner
          </li>
        </ul>
        <p>7/19/2020:</p>
        <ul>
          <li>Bid by clicking on a predefined set of valid bids</li>
        </ul>
        <p>7/18/2020:</p>
        <ul>
          <li>Allow zero-sized kitty in the UI</li>
        </ul>
        <p>7/15/2020:</p>
        <ul>
          <li>Add game option for limiting who can start a game</li>
        </ul>
        <p>7/09/2020:</p>
        <ul>
          <li>
            Add a game option for (disallowing) shadowing of other players
          </li>
        </ul>
        <p>7/02/2020:</p>
        <ul>
          <li>
            (#21) Add a screen and confetti when you successfully defend A!
          </li>
        </ul>
        <p>7/02/2020:</p>
        <ul>
          <li>(#171) Add game option disable taking back bids</li>
          <li>(#68) Add game option disable taking back plays</li>
          <li>
            (#17) Add game option for &ldquo;stealing&rdquo; the bottom cards
          </li>
        </ul>
        <p>7/01/2020:</p>
        <ul>
          <li>Add the option to use SVG cards rather than text cards.</li>
        </ul>
        <p>6/28/2020:</p>
        <ul>
          <li>
            (#163) add game option to reward a bonus level for landlord team to
            win with a smaller size team
          </li>
        </ul>
        <p>6/26/2020:</p>
        <ul>
          <li>(#160) add game option to allow outbid only with more cards</li>
        </ul>
        <p>6/25/2020:</p>
        <ul>
          <li>
            (#158) add user option to display bid cards in separate row in Draw
            stage
          </li>
        </ul>
        <p>6/24/2020:</p>
        <ul>
          <li>
            (#156) add FirstLandlordSelectionPolicy to set the first bidder as
            landlord when no landlord is selected
          </li>
        </ul>
        <p>6/21/2020:</p>
        <ul>
          <li>(#145) Save, load, reset game settings</li>
          <li>(#154) Landlord emoji option</li>
        </ul>
        <p>6/20/2020:</p>
        <ul>
          <li>Add the ability to wrap-around after defending on A.</li>
          <li>Show throw breakdowns in the UI to make throws more obvious.</li>
        </ul>
        <p>6/17/2020:</p>
        <ul>
          <li>Fix bug where points display was highlighted blue.</li>
        </ul>
        <p>6/14/2020:</p>
        <ul>
          <li>Fix bug where previous-trick showed current trick.</li>
          <li>
            (#134) Fix bug where defend-points allowed defending team to skip
            defending points.
          </li>
        </ul>
        <p>6/13/2020:</p>
        <ul>
          <li>
            (#133) Improve trick list to show landlord, better coloring of team
            and winning trick.
          </li>
        </ul>
        <p>6/12/2020:</p>
        <ul>
          <li>
            (#131) Add option to disallow using highest non-trump card to select
            friend.
          </li>
        </ul>
        <p>6/7/2020:</p>
        <ul>
          <li>(#127) Simplify friend selection description.</li>
        </ul>
        <p>6/6/2020:</p>
        <ul>
          <li>
            (#125) Highlight all members of the landlord&apos;s team in the
            trick view.
          </li>
        </ul>
        <p>6/5/2020:</p>
        <ul>
          <li>
            Fix a bug (#35) so as to disallow picking trump cards as friend.
          </li>
        </ul>
        <p>5/25/2020:</p>
        <ul>
          <li>
            Fix bug in longest-tuple-protected mode where tractors of longer
            tuples would erroneously get drawn out.
          </li>
          <li>
            Add support for throw evaluation based on the highest card in the
            throw.
          </li>
        </ul>
        <p>5/24/2020:</p>
        <ul>
          <li>Add game result statistics tracking.</li>
        </ul>
        <p>5/13/2020:</p>
        <ul>
          <li>
            Add an option to protect triples from being drawn out by pairs
          </li>
          <li>Fill in the suit character in the trump UI</li>
        </ul>
        <p>5/8/2020:</p>
        <ul>
          <li>
            When leader is set to winner-of-bid, players bid their own levels
            rather than a random selected one.
          </li>
          <li>Card labels are not visible above the settings pane.</li>
          <li>
            Cards can be revealed from the bottom when the deck is fully drawn
            to determine trump.
          </li>
        </ul>
        <hr />
        <p>Changes prior to 5/8/2020 not listed</p>
      </ReactModal>
    </>
  );
};

const Credits = (): JSX.Element => (
  <p style={{ padding: "0 20px" }}>
    Made by Robert Ying (
    <a href="mailto:me@robertying.com">me@robertying.com</a>), Abra Shen, and
    other{" "}
    <a
      href="https://github.com/rbtying/shengji/graphs/contributors"
      target="_blank"
      rel="noreferrer"
    >
      friends
    </a>
    . Consider buying us boba via Venmo at @Robert-Ying or via{" "}
    <a
      href="https://donate.stripe.com/aEU8x982f3oj4Ja7ss"
      target="_blank"
      rel="noreferrer"
    >
      other payment methods
    </a>
    , or contributing on{" "}
    <a
      href="https://github.com/rbtying/shengji"
      target="_blank"
      rel="noreferrer"
    >
      GitHub
    </a>
    !
    <span style={{ float: "right" }}>
      <ChangeLog />
    </span>
  </p>
);

export default Credits;
