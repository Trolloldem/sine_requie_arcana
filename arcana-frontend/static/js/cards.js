function add_player(name) {
        let html =
                `<div class="card mt-3" id="player_${name}">
                  <h3 class="card-header">${name}</h3>
                  <div class="card-body">
                        <div class="row">
                                <div class="col-sm text-center" id="cover_${name}">
                                        <img src="/arcana/cover.jpg" class="cards">
                                </div>
                                <div class="col-sm text-center" id="card_${name}">
                                </div>
                        </div>
                  </div>`
 
        let new_div = document.createElement('div');
        new_div.setAttribute("class", "container");
        new_div.innerHTML = html;
        document.body.appendChild(new_div);
        
}

function remove_card(name) {
        let arcana = document.getElementById(`arcana_${name}`)
        if (arcana != null) {
                arcana.remove();
        }
}

function remove_cover(name) {
        let cover_div = document.getElementById(`cover_${name}`)
        if (cover_div.children.length === 1) {
                cover_div.children[0].remove();
        }
}

function add_cover(name) {
        let cover_div = document.getElementById(`cover_${name}`);
        if (cover_div.children.length === 0) {
                let new_img = document.createElement("img");
                new_img.setAttribute("href", "#");
                new_img.setAttribute("src", `/arcana/cover.jpg`);
                new_img.setAttribute("class", "cards");
                cover_div.appendChild(new_img);
        }
}

function subscribe(uri) {
  var retryTime = 1;

  function connect(uri) {
    const events = new EventSource(uri);

    events.addEventListener("message", (ev) => {
      let data = JSON.parse(ev.data);
      if (document.getElementById(`player_name_${data.name}`) == null)
      {
              if (!data.is_shuffle) {
                      if (data.arcana == null) {
                        add_player(data.name);
                      } else {
                        add_card(data.name, data.arcana);
                        if (data.is_last_card) {
                                remove_cover(data.name);
                        }
                      }
              } else {
                     remove_card(data.name);
                     add_cover(data.name);
              }
      }
    });

    events.addEventListener("open", () => {
      console.log(`connected to event stream at ${uri}`);
      retryTime = 1;
    });

    events.addEventListener("error", () => {
      events.close();

      let timeout = retryTime;
      retryTime = Math.min(64, retryTime * 2);
      console.log(`connection lost. attempting to reconnect in ${timeout}s`);
      setTimeout(() => connect(uri), (() => timeout * 1000)());
    });
  }

  connect(uri);
}

subscribe("/subscribe");

function add_card(name, card){
        let card_div = document.getElementById(`card_${name}`);
        if (card_div.children.length === 0) {
                let new_img = document.createElement("img");
                new_img.setAttribute("href", "#");
                new_img.setAttribute("src", `/arcana/${card}.jpg`);
                new_img.setAttribute("class", "cards");
                new_img.setAttribute("id", `arcana_${name}`);
                card_div.appendChild(new_img);
        } else {
                let img = document.getElementById(`arcana_${name}`);
                img.setAttribute("src", `/arcana/${card}.jpg`);
        }
}

function get_card(name) {
        fetch('/get')
          .then((response) => response.json())
          .then((data) => {
                if (data.error === null) {
                        add_card(name, data.arcana);
                        if (data.is_last_card) {
                                remove_cover(name);
                        }
                }
          });
}

function shuffle_deck(name) {
        fetch('/shuffle')
          .then((response) => response)
          .then((data) => remove_card(name));
        add_cover(name);
}
