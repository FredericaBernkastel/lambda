'use strict';

$(function(){
  var __path_t    = __glob.path_t,
      __root_url  = __glob.root_url,
      __rpc       = __glob.rpc,
      __cors_h    = __glob.cors_h;
  
  $('a[href="#"]').on('click', function(e){
    e.preventDefault();
  });

  const rpc = {
    Success:      0,
    InvalidLogin: 101
  };

  /* any page
   * ##########################################*/
  {
    var send_mutex = false;

    $('.header .nav-menu .user .logout').on('click', function(){
      if(send_mutex)
        return;
      send_mutex = true;

      var data = {
        'cors_h': __cors_h
      };

      $.ajax({
        type: 'POST',
        url: __rpc + 'auth/logout',
        data: JSON.stringify(data),
        success: function(data){
          send_mutex = false;
          var data = JSON.parse(data);
          if (data.result === rpc.Success)
            window.location.reload(false);
        },
        error: function(jqXHR, status, error){
          send_mutex = false;
        }
      });
    });
  }

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

      if(validate()){
        send_mutex = true;

        self.html(self.attr('data-spinner'));
        var data = {
          'login': $wrapper.find('input#login').prop('value'),
          'password': $wrapper.find('input#password').prop('value'),
          'cors_h': __cors_h
        }

        var default_error   = 'Server error!';

        $.ajax({
          type: 'POST',
          url: __rpc + 'auth/login',
          data: JSON.stringify(data),
          success: function(data){
            send_mutex = false;
            self.html(self.attr('data-html'));
            var data = JSON.parse(data);

            switch(data.result){
              case rpc.InvalidLogin: si_error('Invalid login or password!'); break;
              case rpc.Success: window.location.reload(false); break;
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

  /* /graffiti/add                    
   * ##########################################*/
  if (__path_t === '/graffiti/add') {
    var send_mutex = false;
    var $wrapper = $('.page-graffiti-add');

    $wrapper.find('.actions-wrapper #save').on('click', function(){
      if(send_mutex)
        return;
      send_mutex = true;

      var datetime = function() {
        var date = $wrapper.find('#date').val();
        var time = $wrapper.find('#time').val();
        if (!date) return null;

        datetime = date + 'T' + (time ? time : '00:00:00');
        datetime = Date.parse(datetime) / 1000; // timestamp in seconds;
        if (!datetime) return null;
        return datetime;
      }();

      var gps = function() {
        var data = $wrapper
          .find('#gps')
          .val()
          .split(',')
          .map(function(x){ return +(x.trim()); });

        var e = {lat: null, long: null};
        if(data.length !== 2) return e;
        if(!data[0] || !data[1]) return e;
        return { lat: data[0], long: data[1] };
      }();

      var data = {
        'cors_h': __cors_h,
        'graffiti': {
          'complaint_id': $wrapper.find('#complaint_id').val(),
          'datetime': datetime,
          'shift_time': +$wrapper.find('#shift_time').val(),
          'intervening': $wrapper.find('#intervening').val(),
          'companions': 0,
          'notes': $wrapper.find('#notes').val(),
        },
        'location': {
          'country': $wrapper.find('#country').val(),
          'city': $wrapper.find('#city').val(),
          'street': $wrapper.find('#street').val(),
          'place': $wrapper.find('#place').val(),
          'property': $wrapper.find('#property').val(),
          'gps_long': gps.lat,
          'gps_lat': gps.long
        }
      }

      $.ajax({
        type: 'POST',
        url: __rpc + 'graffiti/add',
        data: JSON.stringify(data),
        success: function(data){
          send_mutex = false;
          var data = JSON.parse(data);

          console.log(JSON.stringify(data, null, 2));

          if (data.result === rpc.Success)
            window.location = __root_url + 'views/graffiti/' + data.id;
        },
        error: function(jqXHR, status, error){
          send_mutex = false;
        }
      });
    })
  }
})