$(function(){
  $('a[href="#"]').on('click', function(e){
    e.preventDefault();
  })

  /* /login                     
   * ##########################################*/
  if (__path_t === '/login') {
    var send_mutex = false;
    var $wrapper = $('.login');

    $wrapper.find('#submit').on('click', function(){
      if(send_mutex)
        return;

      var self = $(this);

      var validate = function(){
        var errors = [];
        if(!$wrapper.find('input#login').prop('value'))
          errors.push('input#login');
        if(!$wrapper.find('input#password').prop('value'))
          errors.push('input#password');
        errors.forEach(function(s){
          $wrapper.find(s).css('border-color', '#ff4d4d');
        });

        return !errors.length;
      }
      var si_error = function(message){
        $wrapper.find('.si-error').html(message).css('display', 'block');
      }
      var default_error = 'Server error!';
      const SUCCESS             = '0';
      const INVALID_QUERY       = '100';
      const INVALID_EMAIL       = '101';
      const INVALID_PASSWORD    = '102';
      const USER_EXISTS         = '103';
      const LOGIN_FAILED        = '104';

      if(validate()){
        send_mutex = true;

        self.html(self.attr('data-spinner'));
        data = {
          'login': $wrapper.find('input#login').prop('value'),
          'password': $wrapper.find('input#password').prop('value'),
          'hash': $wrapper.find('input#hash').prop('value')
        }
        $.ajax({
          type: 'POST',
          url: __root_url + 'rpc/' + self.attr('data-action'),
          data: JSON.stringify(data),
          success: function(data){
            send_mutex = false;
            self.html(self.attr('data-html'));

            switch(data){
              case INVALID_QUERY: si_error(default_error); break;
              case INVALID_EMAIL:
              case INVALID_PASSWORD:
              case LOGIN_FAILED: si_error('Invalid login or password!'); break;
              case SUCCESS: window.location.reload(false); break;
              default: si_error(default_error); break;
            }
          },
          error: function(jqXHR, status, error){
            send_mutex = false;
            self.html(self.attr('data-html'));
            si_error(default_error);
          }
        });
      }
    });

    $wrapper.find('input#login, input#password').on('keydown', function(e){
      if(e.keyCode == 13){
        e.preventDefault();
        $wrapper.find('#submit').trigger('click');
      }
    });
  }
})