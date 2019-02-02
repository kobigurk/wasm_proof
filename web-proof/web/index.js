var worker = new Worker('./worker.js');

function run_generate() {
  $('#spinner_generate').show();
  worker.postMessage({type: 'generate'});
}

window.run_generate = run_generate;

function run_generate_save() {
  var $link = $("<a />");
  var text = window.params;
  $link
    .attr( "download", "dl.zkp" )
    .attr( "href", "data:application/octet-stream," + text )
    .appendTo( "body" )
    .get(0)
    .click();
}

window.run_generate_save = run_generate_save;

function run_generate_load() {
  $('#input_load_params').click();
}

window.run_generate_load = run_generate_load;

function run_prove() {
  $('#spinner_prove').show();
  let params = window.params;
  let x = $('#txt_prove_x').val();
  worker.postMessage({type: 'prove', params, x});
}
window.run_prove = run_prove;

function run_verify() {
  $('#spinner_verify').show();
  let params = window.params;
  let proof = $('#txt_verify_proof').val();
  let h = $('#txt_verify_h').val() ;
  worker.postMessage({type: 'verify', params, proof, h});
}
window.run_verify = run_verify;

let eventHandler = function(event) {
    switch (event.data.type) {
      case 'wasm_loaded':
        $('.spinner_wasm').hide();
        break;
      case 'generate':
      $('#spinner_generate').hide();
      if (event.data.error) {
        let e = event.data.error;
        $('#tr_params').hide();
        $('#btn_save_to_file').hide();
        $('#td_params').text('');
        $('#tr_generate_error').show();
        $('#td_generate_error').text(event.data.error);
      } else {
        let gen = event.data.result;
        window.params = gen.params;
        $('#tr_params').show();
        $('#btn_save_to_file').show();
        $('#td_params').text('Generated, size: ' + Math.round(gen.params.length/1024) + 'kb');
        $('#div_prove_params').addClass('is-dirty');
        $('#txt_prove_params').val('Loaded from memory');
        $('#div_verify_params').addClass('is-dirty');
        $('#txt_verify_params').val('Loaded from memory');
        $('#tr_generate_error').hide();

      }
      $('#table_generate').show();
      break;

      case 'prove':
      $('#spinner_prove').hide();
      if (event.data.error) {
        $('#tr_h').hide();
        $('#td_h').text('');
        $('#tr_proof').hide();
        $('#td_proof').text('');
        $('#tr_prove_error').show();
        $('#td_prove_error').text(event.data.error);
      } else {
        var p = event.data.result;
        $('#tr_h').show();
        $('#tr_proof').show();
        $('#td_h').text(p.h);
        $('#td_proof').text(p.proof);
        $('#div_verify_proof').addClass('is-dirty');
        $('#txt_verify_proof').val(p.proof);
        $('#div_verify_h').addClass('is-dirty');
        $('#txt_verify_h').val(p.h);
        $('#tr_prove_error').hide();
      }
      $('#table_prove').show();
      break;
      case 'verify':
      $('#spinner_verify').hide();
      if (event.data.error) {
        $('#tr_result').hide();
        $('#td_result').text('');
        $('#tr_verify_error').show();
        $('#td_verify_error').text(event.data.error);
      } else {
        let v = event.data.result;
        $('#tr_result').show();
        $('#td_result').text(v.result);
        $('#tr_verify_error').hide();
      }
      $('#table_verify').show();
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
      result: { params: ev.target.result }
    }
  });
};

$field.on("change", function() {
    var file = this.files[0];
    reader.readAsText( file );
});


