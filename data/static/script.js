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

  // i hate dynamic types
  function null_norm(x) {
    if (x === '' || x === undefined || x === NaN)
      return null;
    return x;
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
        if(f.size > 10 * 1024 * 1024){
          display_error('Maximum file size exceeded (10MB)');
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

  //====================== image presentation controller
  function image_presentation($wrapper, img_folder) {
    var $img = $wrapper.children('img');
    var $link_prev = $($wrapper.parents()[1]).find('.link-prev');
    var $link_next = $($wrapper.parents()[1]).find('.link-next');
    var link_state = function(i, length){
      var ret;
      if (i === 0 && length > 1)
        ret = [false, true];
      else if (i > 0 && i < length - 1)
        ret = [true, true];
      else if (i > 0 && i === length - 1)
        ret = [true, false];
      else
        ret = [false, false];
      $link_prev.css('visibility', ret[0] ? 'visible' : 'hidden');
      $link_next.css('visibility', ret[1] ? 'visible' : 'hidden');
    };
    if ($wrapper.children('.images').length) {
      var images = JSON.parse($wrapper.children('.images').html());
      link_state(0, images.length);

      var set_src = function(i) {
        var hash = images[i];
        var path = __root_url + 'static/img/' + img_folder + '/' + hash[0] + hash[1] + '/' + hash + '_p1.jpg';
        $img.attr('src', path);
        $img.attr('data-id', i);
        link_state(i, images.length);
      }

      $link_prev.on('click', function(){
        set_src(+$img.attr('data-id') - 1);
      });
      $link_next.on('click', function(){
        set_src(+$img.attr('data-id') + 1);
      });

      $img.on('click', function(){
        var self = $(this);
        var src = self.attr('src').replace(/_p1\.jpg$/, '_p0.jpg');
        if (src)
          $.fancybox.open({
            src: src
          });
      })
    }
  }

  //====================== map controller
  function map_presentation($wrapper, graffitis /* [{id, thumbnail, coords: [f64; 2]}] */) {
    if (graffitis.length === 0)
      return;

    var api_key = __glob['gmaps_api_key'];
    var iframe = document.createElement("iframe");
    iframe.setAttribute('allowFullScreen', '');
    iframe.onload = function() {
      var doc = iframe.contentDocument;

      iframe.contentWindow.showNewMap = function() {
        var self = this;
        var mapContainer =  doc.createElement('div');
        mapContainer.id = 'map';
        doc.body.appendChild(mapContainer);

        var map = new self.google.maps.Map(mapContainer, {
          //center: new self.google.maps.LatLng(0, 0),
          //zoom: 5,
          mapTypeId: 'roadmap',
          disableDefaultUI: true,
          fullscreenControl: true,
          styles: [
            {
              "featureType": "poi.business",
              "stylers": [
                {
                  "visibility": "off"
                }
              ]
            },
            {
              "featureType": "transit",
              "stylers": [
                {
                  "visibility": "off"
                }
              ]
            }
          ]
        });

        var bounds = new self.google.maps.LatLngBounds();

        var markers = graffitis.map(function(graffiti, i) {
          var latlng = { lat: graffiti.coords[0], lng: graffiti.coords[1] };
          bounds.extend(latlng);
          var marker = new self.google.maps.Marker({
            position: latlng
            //map: map
          });

          if (graffiti.thumbnail) {
            var thumbnail = __root_url + 'static/img/graffiti/' + graffiti.thumbnail[0] + graffiti.thumbnail[1] + '/' + graffiti.thumbnail + '_p2.jpg'
            var infowindow = new self.google.maps.InfoWindow({
              content: '\
                <div id="thumbnail">\
                  <img src="' + thumbnail + '">\
                </div>',
              disableAutoPan: true
            });
            marker.addListener('mouseover', function() {
              infowindow.open(map, marker);
            });
            marker.addListener('mouseout', function() {
              infowindow.close();
            });
          }
          marker.addListener('click', function() {
            window.open(__root_url + 'views/graffiti/' + graffiti.id, '_blank');
          });
          return marker;
        });

        // Add a marker clusterer to manage the markers
        var markerCluster = new self.MarkerClusterer(map, markers, {
          imagePath: __root_url + 'static/img/map/clusters/m',
          gridSize: 30
        });

        { // limit min zoom
          var epsilon = 0.0125;
          var center = bounds.getCenter();
          bounds.extend({lat: center.lat(), lng: center.lng() - epsilon});
          bounds.extend({lat: center.lat(), lng: center.lng() + epsilon});
        }
        
        map.fitBounds(bounds);
      }

      var css = document.createElement('style');
      css.innerHTML = '                                                   \
        body { margin: 0; }                                               \
        #map { width: 100%; height: 100%; }                               \
          #map .gm-style .gm-style-iw-c { padding: 0; border-radius: 0; } \
      ';

      var head = iframe.contentDocument.getElementsByTagName('head')[0];
      head.appendChild(css);

      var script = document.createElement('script');
      script.type = 'text/javascript';
      script.src = __root_url + 'static/markerclustererplus.min.js';
      script.onload = function() {
        var script = document.createElement('script');
        script.type = 'text/javascript';
        script.src = 'https://maps.googleapis.com/maps/api/js?key=' + api_key + '&callback=showNewMap';
        doc.body.appendChild(script);
      }
      doc.body.appendChild(script);
    };

    $wrapper.append(iframe);
  }

  //====================== author input controller
  function author_input($inputs) {
    var author_focus_ctx = function() {
      var self = $(this);
      var select = $('<select />');
      {
        $(self.parents()[1]).after(select);

        select.select2({
          ajax: {
            type: 'POST',
            url: __rpc + 'search/author_names',
            data: function (params) {
              var query = JSON.stringify({
                'cors_h': __cors_h,
                'term': params.term
              });
              return query;
            },
            processResults: function (data) {
              return {
                results: JSON.parse(data).result.map(function(x, i){
                  return {
                    'id': x.id,
                    'text': x.name
                  }
                })
              };
            }
          },
          minimumInputLength: 3
        });

        select.val(self.attr('data-id'));
        select.select2('open');
        select.on('select2:close', function(){
          var data = select.select2('data');
          select.select2('destroy');
          select.remove();
          if (data.length) {
            self.val(data[0].text);
            self.attr('data-id', data[0].id);
            self.trigger('input');
          }
        })
      }
    };

    var author_input_ctx = function() {
      var self = $(this);
      var next = $(self.parents()[1]).next();
      var value = self.val();
      var is_last = next.is('[data-type="x-template"]');

      if (value === '' && !is_last) {
        $(self.parents()[1]).remove();
        //next.find('input[type="text"]').focus();
      }

      if (value !== '' && is_last){
        var tpl = $(
            $(self.parents()[2])
              .find('[data-type="x-template"]')
              .attr('data')
          );
        tpl.find('input[type="text"]')
          .on('focus', author_focus_ctx)
          .on('input', author_input_ctx)
          .siblings('.delete')
          .on('click', author_clear_ctx);
        $(self.parents()[1]).after(tpl);
      }
    };
    var author_clear_ctx = function() {
      var self = $(this);
      self
        .siblings('input[type="text"]')
        .val('')
        .trigger('input');
    };

    $inputs
      .on('focus', author_focus_ctx)
      .on('input', author_input_ctx)
      .siblings('.delete')
      .on('click', author_clear_ctx);
  }

  //====================== graffiti tags input controller
  function graffiti_tags_input($wrapper) {
    $wrapper.select2({
      tags: true,
      ajax: {
        type: 'POST',
        url: __rpc + 'search/tag_names',
        data: function (params) {
          var query = JSON.stringify({
            'cors_h': __cors_h,
            'term': params.term
          });
          return query;
        },
        processResults: function (data) {
          return {
            results: JSON.parse(data).result.map(function(x, i){
              return {
                'id': x.name,
                'text': x.name
              }
            })
          };
        }
      },
      minimumInputLength: 1
    });
  }

  //====================== graffiti search controller
  function graffiti_search($wrapper) {
    $wrapper.find('.actions-wrapper #search').on('click', function(){
      var authors = function(){
        var result = [];
        $wrapper.find('.node108 .row input[data-id]').map(function(i, x){
          var self = $(x);
          result.push({
            id: +self.attr('data-id'),
            indubitable: $(self.parents()[1]).find('input[type="checkbox"]').prop('checked'),
            name: self.val()
          })
        });
        return result;
      }();

      var tags = $wrapper
        .find('.node121_1 .tags-input')
        .select2('data')
        .map(function(x) {
          return x.text;
        });

      var data = JSON.stringify({
        'country': null_norm($wrapper.find('#country').val()),
        'city': null_norm($wrapper.find('#city').val()),
        'street': null_norm($wrapper.find('#street').val()),
        'place': null_norm($wrapper.find('#place').val()),
        'property': null_norm($wrapper.find('#property').val()),
        'date_before': null_norm($wrapper.find('#date_before').val()),
        'date_after': null_norm($wrapper.find('#date_after').val()),
        'authors_number': null_norm(parseInt($wrapper.find('#authors_number').val())),
        'authors': authors,
        'tags': tags
      });

      var data = base64.bytesToBase64(gzip.zip(data, {level: 9})).replace(/\+/g, '-').replace(/\//g, '_');

      window.location = __root_url + 'views/graffitis/search/' + data;
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

  /* /home                     
  * ##########################################*/
  if (__path_t === '/home') {
    var $wrapper = $('.page-home');
    {
      var $map = $wrapper.find('.node104 > .map');
      map_presentation($map, JSON.parse($map.attr('data')));
    }
  }

  /* /graffitis           
   * /graffitis/page/:page    
   * /graffitis/search/:x-data
   * /graffitis/search/:x-data/page/:page      
   * ##########################################*/
  if (__path_t === '/graffitis' || __path_t === '/graffitis/page/:page' || __path_t === '/graffitis/search/:x-data' || __path_t === '/graffitis/search/:x-data/page/:page') {
    $wrapper = $('.page-graffitis');

    // hotkeys
    $(document)
      .on('keydown', null, 'left', function(){
        var link = $wrapper.find('.navigation .n_back a');
        if(link.length)
          link[0].click()
      })
      .on('keydown', null, 'right', function(){
        var link = $wrapper.find('.navigation .n_next a');
        if(link.length)
          link[0].click()
      })

    // search
    {
      var init = false;

      var wrp = $wrapper.find('.search > .wrp');
      $wrapper.find('.search > .title').on('click', function(){
        var self = $(this);
        var icon = self.children('.icon'); 
        wrp.toggle();
        if(wrp.css('display') === 'block') {
          if(!init) {
            // author input controller
            author_input($wrapper.find('.search .node108 .row input[type="text"]'));

            // graffiti tags input controller
            graffiti_tags_input($wrapper.find('.node121_1 .tags-input'));

            init = true;
          }
          icon.html(icon.attr('data-up'));
        }
        else
          icon.html(icon.attr('data-down'));
      });
      if($wrapper.find('.search').hasClass('init'))
         $wrapper.find('.search > .title').trigger('click');

      // graffiti search controller
      graffiti_search($wrapper.find('.search'));
    }
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

      var authors = function(){
        var result = [];
        $wrapper.find('.node108 .row input[data-id]').map(function(i, x){
          var self = $(x);
          result.push({
            id: +self.attr('data-id'),
            indubitable: $(self.parents()[1]).find('input[type="checkbox"]').prop('checked')
          })
        });
        return result;
      }();

      var tags = $wrapper
        .find('.node112 .tags-input')
        .select2('data')
        .map(function(x) {
          return x.text;
        });

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
        },
        'authors': authors,
        'tags': tags
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
    });

    // authors input controller
    author_input($wrapper.find('.search .node108 .row input[type="text"]'));

    // graffiti tags input controller
    graffiti_tags_input($wrapper.find('.node112 .tags-input'));
  }

  /* /graffiti/:id               
   * ##########################################*/
  if (__path_t === '/graffiti/:id') {
    var send_mutex = false;
    var $wrapper = $('.page-graffiti');

    image_presentation($wrapper.find('.graffiti-image'), 'graffiti');
    {
      var $map = $wrapper.find('.node106 > .map');
      map_presentation($map, JSON.parse($map.attr('data')));
    }

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

  /* /authors           
   * /authors/page/:page          
   * ##########################################*/
  if (__path_t === '/authors' || __path_t === '/authors/page/:page') {
    var $wrapper = $('.page-authors');

    // hotkeys
    $(document)
      .on('keydown', null, 'left', function(){
        var link = $('.page-authors .navigation .n_back a');
        if(link.length)
          link[0].click()
      })
      .on('keydown', null, 'right', function(){
        var link = $('.page-authors .navigation .n_next a');
        if(link.length)
          link[0].click()
      });

    // search
    {
      var init = false;

      var wrp = $wrapper.find('.search > .wrp');
      $wrapper.find('.search > .title').on('click', function(){
        var self = $(this);
        var icon = self.children('.icon'); 
        wrp.toggle();
        if(wrp.css('display') === 'block') {
          if(!init) {
            // authors input controller
            author_input($wrapper.find('.search .node108 .row input[type="text"]'));

            // active_in locations input controller
            $wrapper.find('.search .node124_2 .tags-input').select2({
              tags: true
            });

            init = true;
          }
          icon.html(icon.attr('data-up'));
        }
        else
          icon.html(icon.attr('data-down'));
      });
      if($wrapper.find('.search').hasClass('init'))
         $wrapper.find('.search > .title').trigger('click');
    }
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

    image_presentation($wrapper.find('.author-image'), 'author');
    {
      var $map = $wrapper.find('.node114_3 > .map');
      map_presentation($map, JSON.parse($map.attr('data')));
    }

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