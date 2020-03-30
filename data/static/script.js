'use strict';

$(function(){
  var __path_t    = __glob.path_t,
      __root_url  = __glob.root_url,
      __rpc       = __glob.rpc,
      __cors_h    = __glob.cors_h;

  class Mutex {
    constructor(onGet, onRelease){
      this.value = false;
      this.onGet = onGet;
      this.onRelease = onRelease;
    }

    get(){
      if(this.value) return false; 
      this.value = true;
      if(typeof this.onGet === 'function') this.onGet(); 
      return true;
    }
    release(){
      if(!this.value) return false; 
      this.value = false;
      if(typeof this.onRelease === 'function') this.onRelease(); 
      return true;
    }
  }

  const rpc = {
    Success:        0,
    InternalError:  100,
    InvalidLogin:   101,
    InvalidRequest: 102
  };

  //====================== popups
  function display_error(message){
    var popup = $('.popup-wrapper#error');
    popup.find('.message').html(message);
    popup.find('.action-btn#close')
      .off('click')
      .on('click', function(){
        popup.css('display', 'none');
      });
    popup.css('display', 'flex');
    return popup;
  };

  function display_warning(message, callback){
    var popup = $('.popup-wrapper#warning');
    popup.find('.message').html(message);
    popup.find('.action-btn#cancel')
      .off('click')
      .on('click', function(){
        popup.css('display', 'none');
      });
    popup.find('.action-btn#ok')
      .off('click')
      .on('click', function(){
        var result = callback();
        if(result !== false)
          popup.css('display', 'none');
      });
    popup.css('display', 'flex');
    return popup;
  }

  //====================== images upload
  function image_upload_ctr(__rpc_fn, $wrapper) {

    var mutex = new Mutex();

    // images iface controller
    var image_controls = function(self){
      function check_class(self){
        return self.hasClass('image') && !self.hasClass('processing') && !self.hasClass('add');
      }
      if(!check_class(self))
        return;

      self.find('.controls .sh .shl').on('click', function(){
        var prev = self.prev();
        if(prev.length)
          if(check_class(prev))
            self.insertBefore(prev);
      });
      self.find('.controls .sh .shr').on('click', function(){
        var next = self.next();
        if(next.length)
          if(check_class(next))
            self.insertAfter(next);
      });
      self.find('.controls .del').on('click', function(){
        self.remove();
      });
    }

    $wrapper.children('.image:not(.processing):not(.add)').each(function(i){
      image_controls($(this));
    });

    $wrapper.find('.image.add').on('click', function(){
      if(mutex.value)
        return;
      $wrapper.find('input#openfiledlg').trigger('click');
    });

    $wrapper.find('input#openfiledlg').on('change', function(evt){
      var files = evt.target.files; // FileList object
      if(files.length === 0)
        return;

      var self = this;

      mutex.get();

      // check file size
      for(var i = 0, f; f = files[i]; i++)
        if(f.size > 1024 * 1024){
          display_error('Maximum file size exceeded (1MB)');
          mutex.release();
          return
        }

      for (var i = 0, f; f = files[i]; i++)
        if (!(f.type === 'image/jpeg')){
          display_error('Unsupported image type');
          mutex.release();
          return;
        }


      var eventCounter = 0;
      var eventResults = [];

      // Loop through the FileList
      for (var i = 0, f; f = files[i]; i++) {
        var reader = new FileReader();
        reader.onload = (function(i) {
          return function(e) {
            eventCounter++;
            eventResults[i] = e.target.result;
            if(eventCounter === files.length) promise_all();
          };
        })(i);
        reader.onerror = function() { display_error('Unable to read file.'); mutex.release(); self.value = ''; };

        reader.readAsDataURL(f);
      }

      var promise_all = function(){

        self.value = '';

        // add thumbnails
        eventResults.forEach(function(file, i, array){
          var $tpl = $($wrapper.find('.image.add *[data-type="x-template"]').attr('data'));
          $tpl.find('img').get(0).src = file;
          $tpl
            .addClass('processing')
            .insertBefore($wrapper.children('.image.add'));
          array[i] = {ref: $tpl, data: array[i]};
        });

        // recursive upload sequence
        function upload(i){
         $.ajax({
            type: 'POST',
            url: __rpc + __rpc_fn,
            data: JSON.stringify({
              'cors_h': __cors_h,
              'data': eventResults[i].data
            }),

            success: function(data){
              function _finally(){
                mutex.release();
                $wrapper.find('.image.processing').remove();
              };
              var data = JSON.parse(data);
              switch(data.result){
                case rpc.InvalidRequest:
                case rpc.InternalError: {
                  display_error('Image upload failed (server error)');
                  _finally();
                  break;
                };
                case rpc.Success: {
                  eventResults[i].ref
                    .attr('data-id', data['temp_id'])
                    .removeClass('processing');
                  image_controls(eventResults[i].ref);

                  if(i < eventResults.length - 1)
                    upload(i + 1);
                  else
                    mutex.release();
                  break;
                }
              }
            },
            error: function(jqXHR, status, error){
              display_error('error: network failure');
            }
          });
        }
        upload(0);
      }
    });
  }
  
  $('a[href="#"]').on('click', function(e){
    e.preventDefault();
  });

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
   * /graffiti/:id/edit                
   * ##########################################*/
  if (__path_t === '/graffiti/add' || __path_t === '/graffiti/:id/edit') {
    var send_mutex = false;
    var $wrapper = $('.page-graffiti-add');

    var __rpc_fn;
    switch (__path_t) {
      case '/graffiti/add':      __rpc_fn = __rpc + 'graffiti/add'; break;
      case '/graffiti/:id/edit': __rpc_fn = __rpc + 'graffiti/edit'; break;
    }

    image_upload_ctr('graffiti/store_image', $wrapper.find('.img_upload_wrp'));

    $wrapper.find('.actions-wrapper #save').on('click', function(){
      if(send_mutex)
        return;
      send_mutex = true;

      var datetime = function() {
        var date = $wrapper.find('#date').val();
        var time = $wrapper.find('#time').val();
        if (!date) return null;

        datetime = date + 'T' + (time ? time : '00:00:00') + 'Z';
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
          'gps_long': gps.long,
          'gps_lat': gps.lat
        }
      }

      if (__path_t === '/graffiti/:id/edit')
        data['graffiti']['id'] = +__glob.data['id'];

      data['images'] = [];
        $wrapper.find('.img_upload_wrp > .image:not(.processing):not(.add)').each(function(){
          var id = $(this).attr('data-id');
          if(id)
            data['images'].push(id);
        });

      $.ajax({
        type: 'POST',
        url: __rpc_fn,
        data: JSON.stringify(data),
        success: function(response){
          send_mutex = false;
          var response = JSON.parse(response);

          if (response.result === rpc.Success){
            var id;
            if (__path_t === '/graffiti/:id/edit')
              id = +__glob.data['id'];
            else
              id = response.id;
            window.location = __root_url + 'views/graffiti/' + id;
          }
        },
        error: function(jqXHR, status, error){
          send_mutex = false;
        }
      });
    })
  }

  /* /graffiti/:id               
   * ##########################################*/
  if (__path_t === '/graffiti/:id') {
    var send_mutex = false;
    var $wrapper = $('.page-graffiti');

    $wrapper.find('.actions-wrapper #delete').on('click', function(){
      display_warning('Delete graffiti?', function(){
        var send_mutex = true;
        var data = {
          'cors_h': __cors_h,
          'id': +__glob.data['id']
        };

        $.ajax({
          type: 'POST',
          url: __rpc + 'graffiti/delete',
          data: JSON.stringify(data),
          success: function(response){
            send_mutex = false;
            var response = JSON.parse(response);

            if (response.result === rpc.Success)
              window.location = __root_url + 'views/graffitis';
          },
          error: function(jqXHR, status, error){
            send_mutex = false;
          }
        });
      });
    });
  }

  /* /author/add    
   * /author/:id/edit                
   * ##########################################*/
  if (__path_t === '/author/add' || __path_t === '/author/:id/edit') {
    var send_mutex = false;
    var $wrapper = $('.page-author-add');

    var __rpc_fn;
    switch (__path_t) {
      case '/author/add':      __rpc_fn = __rpc + 'author/add'; break;
      case '/author/:id/edit': __rpc_fn = __rpc + 'author/edit'; break;
    }

    image_upload_ctr('author/store_image', $wrapper.find('.img_upload_wrp'));

    $wrapper.find('.actions-wrapper #save').on('click', function(){
      if(send_mutex)
        return;

      var validate = function(){
        var errors = [];
        if(!$wrapper.find('#name').val())
          errors.push('#name');
        errors.forEach(function(s){
          $wrapper.find(s).css('border-color', '#ff4d4d');
        });
        return !errors.length;
      }

      if (!validate())
        return;

      send_mutex = true;

      var data = {
        'cors_h': __cors_h,
        'name': $wrapper.find('#name').val(),
        'age': +$wrapper.find('#age').val(),
        'height': +$wrapper.find('#height').val(),
        'handedness': +$wrapper.find('#handedness').val(),
        'home_city': $wrapper.find('#home_city').val(),
        'social_networks': $wrapper.find('#social_networks').val(),
        'notes': $wrapper.find('#notes').val()
      };

      if (data['age'] === 0) data['age'] = null;
      if (data['height'] === 0) data['height'] = null;

      if (__path_t === '/author/:id/edit')
        data['id'] = +__glob.data['id'];

      data['images'] = [];
        $wrapper.find('.img_upload_wrp > .image:not(.processing):not(.add)').each(function(){
          var id = $(this).attr('data-id');
          if(id)
            data['images'].push(id);
        });

      $.ajax({
        type: 'POST',
        url: __rpc_fn,
        data: JSON.stringify(data),
        success: function(response){
          send_mutex = false;
          var response = JSON.parse(response);

          if (response.result === rpc.Success){
            var id;
            if (__path_t === '/author/:id/edit')
              id = +__glob.data['id'];
            else
              id = response.id;
            window.location = __root_url + 'views/author/' + id;
          }
        },
        error: function(jqXHR, status, error){
          send_mutex = false;
        }
      });
    })
  }

  /* /author/:id               
   * ##########################################*/
  if (__path_t === '/author/:id') {
    var send_mutex = false;
    var $wrapper = $('.page-author');

    $wrapper.find('.actions-wrapper #delete').on('click', function(){
      display_warning('Delete author?', function(){
        var send_mutex = true;
        var data = {
          'cors_h': __cors_h,
          'id': +__glob.data['id']
        };

        $.ajax({
          type: 'POST',
          url: __rpc + 'author/delete',
          data: JSON.stringify(data),
          success: function(response){
            send_mutex = false;
            var response = JSON.parse(response);

            if (response.result === rpc.Success)
              window.location = __root_url + 'views/authors';
          },
          error: function(jqXHR, status, error){
            send_mutex = false;
          }
        });
      });
    });
  }

})