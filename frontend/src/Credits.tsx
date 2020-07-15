import * as React from "react";
import * as ReactModal from "react-modal";

const contentStyle = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
};

const ChangeLog = (): JSX.Element => {
  const [modalOpen, setModalOpen] = React.useState<boolean>(false);
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
        onRequestClose={() => setModalOpen(false)}
        shouldCloseOnOverlayClick
        shouldCloseOnEsc
        style={{ content: contentStyle }}
      >
        <h2>Change Log</h2>
        <p>7/15/2020:</p>
        <ul>
          <li>add game option for starting a game</li>
        </ul>
        <p>7/09/2020:</p>
        <ul>
          <li>game option for game shadowing</li>
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
  <p>
    Made by Robert Ying, Abra Shen, and other{" "}
    <a
      href="https://github.com/rbtying/shengji/graphs/contributors"
      target="_blank"
      rel="noreferrer"
    >
      friends
    </a>
    . Consider buying us boba via Venmo at @Robert-Ying, or contributing on{" "}
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
