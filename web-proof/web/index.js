var worker = new Worker('./worker.js');

function run_generate() {
  $('#spinner_generate').show();
  worker.postMessage({type: 'generate', circuit: 'dl'});
}

window.run_generate = run_generate;

function tree_run_generate() {
  $('#tree_spinner_generate').show();
  let depth = $('#tree_txt_depth').val();
  worker.postMessage({type: 'generate', circuit: 'tree', depth});
}

window.tree_run_generate = tree_run_generate;

function run_generate_save() {
  var $link = $("<a />");
  var text = window.params;
  $link
    .attr( "download", "dl.zkp" )
    .attr( "href", URL.createObjectURL(new Blob([text], {type: 'data:application/octet-stream'})))
    .appendTo( "body" )
    .get(0)
    .click();
}

window.run_generate_save = run_generate_save;

function tree_run_generate_save() {
  var $link = $("<a />");
  var text = window.params;
  $link
    .attr( "download", "tree.zkp" )
    .attr( "href", URL.createObjectURL(new Blob([text], {type: 'data:application/octet-stream'})))
    .appendTo( "body" )
    .get(0)
    .click();
}

window.tree_run_generate_save = tree_run_generate_save;



function run_generate_load() {
  $('#input_load_params').click();
}

window.run_generate_load = run_generate_load;

function tree_run_generate_load() {
  $('#tree_input_load_params').click();
}

window.tree_run_generate_load = tree_run_generate_load;

function run_prove() {
  $('#spinner_prove').show();
  let params = window.params;
  let x = $('#txt_prove_x').val();
  worker.postMessage({type: 'prove', params, x, circuit: 'dl'});
}
window.run_prove = run_prove;

function tree_run_prove() {
  $('#tree_spinner_prove').show();
  let params = window.params;
  let x = $('#tree_txt_prove_x').val();
  let depth = $('#tree_txt_prove_depth').val();
  worker.postMessage({type: 'prove', params, x, circuit: 'tree', depth});
}
window.tree_run_prove = tree_run_prove;



function run_verify() {
  $('#spinner_verify').show();
  let params = window.params;
  let proof = $('#txt_verify_proof').val();
  let h = $('#txt_verify_h').val() ;
  worker.postMessage({type: 'verify', params, proof, h, circuit: 'dl'});
}
window.run_verify = run_verify;

function tree_run_verify() {
  $('#tree_spinner_verify').show();
  let params = window.params;
  let proof = $('#tree_txt_verify_proof').val();
  let h = $('#tree_txt_verify_h').val() ;
  worker.postMessage({type: 'verify', params, proof, h, circuit: 'tree'});
}
window.tree_run_verify = tree_run_verify;



let eventHandler = function(event) {
    let pref = '';
    if (event.data.circuit == 'tree') {
      pref = 'tree_';
    }
    switch (event.data.type) {
      case 'wasm_loaded':
        $('.spinner_wasm').hide();
        break;
      case 'generate':
      $('#' + pref + 'spinner_generate').hide();
      if (event.data.error) {
        let e = event.data.error;
        $('#' + pref + 'tr_params').hide();
        $('#' + pref + 'btn_save_to_file').hide();
        $('#' + pref + 'td_params').text('');
        $('#' + pref + 'tr_generate_error').show();
        $('#' + pref + 'td_generate_error').text(event.data.error);
      } else {
        let gen = event.data.result;
        window.params = gen.params;
        $('#' + pref + 'tr_params').show();
        $('#' + pref + 'btn_save_to_file').show();
        $('#' + pref + 'td_params').text('Generated, size: ' + Math.round(gen.params.length/1024) + 'kb');
        $('#' + pref + 'div_prove_params').addClass('is-dirty');
        $('#' + pref + 'txt_prove_params').val('Loaded from memory');
        $('#' + pref + 'div_verify_params').addClass('is-dirty');
        $('#' + pref + 'txt_verify_params').val('Loaded from memory');
        if (event.data.circuit == 'tree') {
          $('#' + pref + 'div_prove_depth').addClass('is-dirty');
          $('#' + pref + 'txt_prove_depth').val($('#' + pref + 'txt_depth').val());
        }
        $('#' + pref + 'tr_generate_error').hide();
        console.log('generate time elapsed: ' + gen.millis);
      }
      $('#' + pref + 'table_generate').show();
      break;

      case 'prove':
      $('#' + pref + 'spinner_prove').hide();
      if (event.data.error) {
        $('#' + pref + 'tr_h').hide();
        $('#' + pref + 'td_h').text('');
        $('#' + pref + 'tr_proof').hide();
        $('#' + pref + 'td_proof').text('');
        $('#' + pref + 'tr_prove_error').show();
        $('#' + pref + 'td_prove_error').text(event.data.error);
      } else {
        var p = event.data.result;
        $('#' + pref + 'tr_h').show();
        $('#' + pref + 'tr_proof').show();
        $('#' + pref + 'td_h').text(p.h);
        $('#' + pref + 'td_proof').text(p.proof);
        $('#' + pref + 'div_verify_proof').addClass('is-dirty');
        $('#' + pref + 'txt_verify_proof').val(p.proof);
        $('#' + pref + 'div_verify_h').addClass('is-dirty');
        $('#' + pref + 'txt_verify_h').val(p.h);
        $('#' + pref + 'tr_prove_error').hide();
        console.log('prove time elapsed: ' + p.millis);
      }
      $('#' + pref + 'table_prove').show();
      break;
      case 'verify':
      $('#' + pref + 'spinner_verify').hide();
      if (event.data.error) {
        $('#' + pref + 'tr_result').hide();
        $('#' + pref + 'td_result').text('');
        $('#' + pref + 'tr_verify_error').show();
        $('#' + pref + 'td_verify_error').text(event.data.error);
      } else {
        let v = event.data.result;
        $('#' + pref + 'tr_result').show();
        $('#' + pref + 'td_result').text(v.result);
        $('#' + pref + 'tr_verify_error').hide();
        console.log('verify time elapsed: ' + v.millis);
      }
      $('#' + pref + 'table_verify').show();
      break;
    }
};

worker.addEventListener('message', eventHandler);

var $field = $("#input_load_params");
var reader = new FileReader();
reader.onload = function( ev ) {
  eventHandler({
    data: {
      type: 'generate',
      circuit: 'dl',
      result: { params: ev.target.result }
    }
  });
};

$field.on("change", function() {
    var file = this.files[0];
    reader.readAsText( file );
});

var $tree_field = $("#tree_input_load_params");
var tree_reader = new FileReader();
tree_reader.onload = function( ev ) {
  eventHandler({
    data: {
      type: 'generate',
      circuit: 'tree',
      result: { params: ev.target.result }
    }
  });
};

$tree_field.on("change", function() {
    var file = this.files[0];
    tree_reader.readAsText( file );
});

