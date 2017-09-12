/*
 * Copyright 2017 Jacob Lifshay
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 */
#include <iostream>
#include <SDL.h>
#include <SDL_syswm.h>
#include <X11/Xlib-xcb.h>
#include <xcb/xcb.h>
#include <xcb/shm.h>
#include <xcb/present.h>
#include <stdexcept>
#include <memory>
#include <cstring>
#include <cstdlib>
#include <cassert>
#include <list>
#include <utility>

#ifndef SDL_VIDEO_DRIVER_X11
#error SDL was not built with X11 support
#endif

class Image_presenter;

struct Image
{
    std::shared_ptr<void> pixels;
    std::size_t row_pitch;
    std::size_t width;
    std::size_t height;
    std::size_t pixel_size;
    std::uint32_t red_mask;
    std::uint32_t green_mask;
    std::uint32_t blue_mask;
    std::uint32_t alpha_mask;
    Image(std::shared_ptr<void> pixels,
          std::size_t row_pitch,
          std::size_t width,
          std::size_t height,
          std::size_t pixel_size,
          std::uint32_t red_mask,
          std::uint32_t green_mask,
          std::uint32_t blue_mask,
          std::uint32_t alpha_mask) noexcept : pixels(std::move(pixels)),
                                               row_pitch(row_pitch),
                                               width(width),
                                               height(height),
                                               pixel_size(pixel_size),
                                               red_mask(red_mask),
                                               green_mask(green_mask),
                                               blue_mask(blue_mask),
                                               alpha_mask(alpha_mask)
    {
    }
};

class Image_presenter
{
    Image_presenter(const Image_presenter &) = delete;
    Image_presenter &operator=(const Image_presenter &) = delete;

private:
    template <typename T = void>
    struct Free_functor
    {
        void operator()(T *p) noexcept
        {
            std::free(p);
        }
    };
    typedef std::unique_ptr<xcb_query_extension_reply_t, Free_functor<xcb_query_extension_reply_t>>
        Query_extension_reply;
    typedef std::unique_ptr<xcb_get_geometry_reply_t, Free_functor<xcb_get_geometry_reply_t>>
        Get_geometry_reply;
    typedef std::unique_ptr<xcb_get_window_attributes_reply_t,
                            Free_functor<xcb_get_window_attributes_reply_t>>
        Get_window_attributes_reply;
    typedef std::unique_ptr<xcb_query_tree_reply_t, Free_functor<xcb_query_tree_reply_t>>
        Query_tree_reply;
    struct Presentable_image : public Image
    {
        Presentable_image(std::shared_ptr<void> pixels,
                          std::size_t row_pitch,
                          std::size_t width,
                          std::size_t height,
                          std::size_t pixel_size,
                          std::uint32_t red_mask,
                          std::uint32_t green_mask,
                          std::uint32_t blue_mask,
                          std::uint32_t alpha_mask)
            : Image(std::move(pixels),
                    row_pitch,
                    width,
                    height,
                    pixel_size,
                    red_mask,
                    green_mask,
                    blue_mask,
                    alpha_mask)
        {
        }
    };
    template <typename Id_type,
              xcb_void_cookie_t (*free_function)(xcb_connection_t *connection, Id_type id)>
    class Server_object
    {
    private:
        Id_type value;
        xcb_connection_t *connection;

    public:
        constexpr Server_object() noexcept : value(), connection()
        {
        }
        constexpr Server_object(std::nullptr_t) noexcept : value(), connection()
        {
        }
        constexpr Server_object(Id_type value, xcb_connection_t *connection) noexcept
            : value(value),
              connection(connection)
        {
            assert(connection);
        }
        void swap(Server_object &other) noexcept
        {
            using std::swap;
            swap(value, other.value);
            swap(connection, other.connection);
        }
        Server_object(Server_object &&rt) noexcept : value(), connection()
        {
            swap(rt);
        }
        Server_object &operator=(Server_object rt) noexcept
        {
            swap(rt);
            return *this;
        }
        ~Server_object() noexcept
        {
            if(connection)
                free_function(connection, value);
        }
        Id_type get() const noexcept
        {
            return value;
        }
    };
    typedef Server_object<xcb_gcontext_t, &xcb_free_gc> Gc;

public:
    class Image_handle
    {
        friend class Image_presenter;

    private:
        std::list<Presentable_image>::iterator iter;
        explicit Image_handle(std::list<Presentable_image>::iterator iter) : iter(std::move(iter))
        {
        }

    public:
        Image_handle() : iter()
        {
        }
        const Image *get() const
        {
            return &*iter;
        }
    };

private:
    xcb_connection_t *const connection;
    const xcb_window_t window;
    const std::size_t image_count;
    std::list<Presentable_image> free_list;
    std::list<Presentable_image> filling_list;
    std::list<Presentable_image> presenting_list;
    bool shm_is_supported;
    Gc gc;
    unsigned window_depth;

private:
    static xcb_query_extension_cookie_t query_extension(xcb_connection_t *connection,
                                                        const char *extension_name) noexcept
    {
        return xcb_query_extension(connection, std::strlen(extension_name), extension_name);
    }

public:
    Image_presenter(xcb_connection_t *connection, xcb_window_t window, std::size_t image_count)
        : connection(connection),
          window(window),
          image_count(image_count),
          free_list(),
          filling_list(),
          presenting_list(),
          shm_is_supported(false),
          gc(),
          window_depth()
    {
        auto mit_shm_cookie = query_extension(connection, "MIT-SHM");
        auto get_geometry_cookie = xcb_get_geometry(connection, window);
        auto get_window_attributes_cookie = xcb_get_window_attributes(connection, window);
        auto query_tree_cookie = xcb_query_tree(connection, window);
        auto gc_id = xcb_generate_id(connection);
        const std::uint32_t gc_params[1] = {
            0, // value for XCB_GC_GRAPHICS_EXPOSURES
        };
        xcb_create_gc(connection, gc_id, window, XCB_GC_GRAPHICS_EXPOSURES, gc_params);
        gc = Gc(gc_id, connection);
        auto mit_shm_reply =
            Query_extension_reply(xcb_query_extension_reply(connection, mit_shm_cookie, nullptr));
        shm_is_supported = mit_shm_reply && mit_shm_reply->present;
        auto get_geometry_reply =
            Get_geometry_reply(xcb_get_geometry_reply(connection, get_geometry_cookie, nullptr));
        if(!get_geometry_reply)
            throw std::runtime_error("xcb_get_geometry failed to reply");
        std::size_t image_width = get_geometry_reply->width;
        std::size_t image_height = get_geometry_reply->height;
        auto get_window_attributes_reply = Get_window_attributes_reply(
            xcb_get_window_attributes_reply(connection, get_window_attributes_cookie, nullptr));
        if(!get_window_attributes_reply)
            throw std::runtime_error("xcb_get_window_attributes failed to reply");
        auto window_visual_id = get_window_attributes_reply->visual;
        auto query_tree_reply =
            Query_tree_reply(xcb_query_tree_reply(connection, query_tree_cookie, nullptr));
        if(!query_tree_reply)
            throw std::runtime_error("xcb_query_tree failed to reply");
        auto root_window = query_tree_reply->root;
        xcb_screen_t *screen = nullptr;
        for(auto iter = xcb_setup_roots_iterator(xcb_get_setup(connection)); iter.rem;
            xcb_screen_next(&iter))
        {
            if(iter.data->root == root_window)
            {
                screen = iter.data;
                break;
            }
        }
        if(!screen)
            throw std::runtime_error("screen not found");
        xcb_visualtype_t *window_visual_type = nullptr;
        for(auto depth_iter = xcb_screen_allowed_depths_iterator(screen); depth_iter.rem;
            xcb_depth_next(&depth_iter))
        {
            for(auto visual_iter = xcb_depth_visuals_iterator(depth_iter.data); visual_iter.rem;
                xcb_visualtype_next(&visual_iter))
            {
                if(visual_iter.data->visual_id == window_visual_id)
                {
                    window_visual_type = visual_iter.data;
                    window_depth = depth_iter.data->depth;
                    break;
                }
            }
            if(window_visual_type)
                break;
        }
        if(!window_visual_type)
            throw std::runtime_error("visual not found");
        std::uint32_t red_mask = window_visual_type->red_mask;
        std::uint32_t green_mask = window_visual_type->green_mask;
        std::uint32_t blue_mask = window_visual_type->blue_mask;
        std::uint32_t alpha_mask;
        switch(window_depth)
        {
        case 24:
            alpha_mask = 0;
            break;
        case 32:
            alpha_mask = ~(red_mask | green_mask | blue_mask);
            break;
        default:
            throw std::runtime_error("unsupported window depth");
        }
        xcb_format_t *window_pixmap_format = nullptr;
        for(auto iter = xcb_setup_pixmap_formats_iterator(xcb_get_setup(connection)); iter.rem;
            xcb_format_next(&iter))
        {
            if(iter.data->depth == window_depth)
            {
                window_pixmap_format = iter.data;
                break;
            }
        }
        if(!window_pixmap_format)
            throw std::runtime_error("pixmap format not found");
        std::size_t image_pixel_size;
        switch(window_pixmap_format->bits_per_pixel)
        {
        case 24:
            image_pixel_size = 3;
            break;
        case 32:
            image_pixel_size = 4;
            break;
        default:
            throw std::runtime_error("unsupported pixmap format bits-per-pixel");
        }
        std::size_t scanline_alignment;
        switch(window_pixmap_format->scanline_pad)
        {
        case 8:
            scanline_alignment = 1;
            break;
        case 16:
            scanline_alignment = 2;
            break;
        case 32:
            scanline_alignment = 4;
            break;
        default:
            throw std::runtime_error("invalid pixmap format scanline-pad");
        }
        std::size_t unpadded_scanline_size = image_pixel_size * image_width;
        std::size_t padded_scanline_size =
            (unpadded_scanline_size + scanline_alignment - 1U) & ~(scanline_alignment - 1U);
        std::size_t image_size = padded_scanline_size * image_height;
        for(std::size_t i = 0; i < image_count; i++)
        {
#warning implement using shared memory
            auto pixels = std::shared_ptr<unsigned char>(new unsigned char[image_size],
                                                         [](unsigned char *p) noexcept
                                                         {
                                                             delete[] p;
                                                         });
            free_list.push_back(Presentable_image(pixels,
                                                  padded_scanline_size,
                                                  image_width,
                                                  image_height,
                                                  image_pixel_size,
                                                  red_mask,
                                                  green_mask,
                                                  blue_mask,
                                                  alpha_mask));
        }
#warning finish implementing Image_presenter::Image_presenter
    }
    Image_handle get_next_image()
    {
        while(true)
        {
            if(!free_list.empty())
            {
                Image_handle retval(free_list.begin());
                filling_list.splice(filling_list.end(), free_list, retval.iter);
                return retval;
            }
            throw std::runtime_error("Image_presenter::get_next_image is not implemented");
#warning finish implementing Image_presenter::get_next_image
        }
    }
    void present_image(Image_handle image_handle)
    {
        assert(image_handle.iter == filling_list.begin() && "images presented out of order");
        presenting_list.splice(presenting_list.end(), filling_list, image_handle.iter);
        auto &image = *image_handle.iter;
#warning implement using shared memory
        std::size_t image_size = image.height * image.row_pitch;
        assert(static_cast<std::uint32_t>(image_size) == image_size);
        xcb_put_image(connection,
                      XCB_IMAGE_FORMAT_Z_PIXMAP,
                      window,
                      gc.get(),
                      image.width,
                      image.height,
                      0,
                      0,
                      0,
                      window_depth,
                      image_size,
                      static_cast<const std::uint8_t *>(image.pixels.get()));
        // we don't have to keep the memory unmodified for xcb, so we move the image to the free
        // list right away
        free_list.splice(free_list.end(), presenting_list, image_handle.iter);
#warning finish implementing Image_presenter::present_image
    }
};

constexpr std::uint32_t get_lowest_set_bit(std::uint32_t v)
{
    return v & -v;
}

std::uint32_t rgb(const Image *image, std::uint8_t r, std::uint8_t g, std::uint8_t b)
{
    return r * get_lowest_set_bit(image->red_mask) | g * get_lowest_set_bit(image->green_mask)
           | b * get_lowest_set_bit(image->blue_mask);
}

int main()
{
    if(SDL_Init(SDL_INIT_VIDEO) < 0)
    {
        std::cerr << "SDL_Init failed: " << SDL_GetError() << std::endl;
        return 1;
    }
    struct Shutdown_sdl
    {
        ~Shutdown_sdl()
        {
            SDL_Quit();
        }
    } shutdown_sdl;
    auto *window = SDL_CreateWindow("XCB Present Test",
                                    SDL_WINDOWPOS_UNDEFINED,
                                    SDL_WINDOWPOS_UNDEFINED,
                                    1024,
                                    768,
                                    SDL_WINDOW_SHOWN);
    if(!window)
    {
        std::cerr << "SDL_CreateWindow failed: " << SDL_GetError() << std::endl;
        return 1;
    }
    struct Window_destroyer
    {
        SDL_Window *window;
        ~Window_destroyer()
        {
            SDL_DestroyWindow(window);
        }
    } window_destroyer{window};
    SDL_SysWMinfo wm_info{};
    SDL_VERSION(&wm_info.version);
    if(!SDL_GetWindowWMInfo(window, &wm_info))
    {
        std::cerr << "SDL_GetWindowWMInfo failed: " << SDL_GetError() << std::endl;
        return 1;
    }
    if(wm_info.subsystem != SDL_SYSWM_X11)
    {
        std::cerr << "SDL window is not an X11 window" << std::endl;
        return 1;
    }
    try
    {
        std::size_t image_count = 3;
        Image_presenter image_presenter(XGetXCBConnection(wm_info.info.x11.display),
                                        static_cast<xcb_window_t>(wm_info.info.x11.window),
                                        image_count);
        SDL_Event event;
        auto last_fps_report_ticks = SDL_GetTicks();
        std::size_t frame_count = 0;
        while(true)
        {
            while(SDL_PollEvent(&event))
            {
                switch(event.type)
                {
                case SDL_QUIT:
                    return 0;
                case SDL_KEYDOWN:
                    if(event.key.keysym.sym == SDLK_ESCAPE
                       || (event.key.keysym.sym == SDLK_F4
                           && (event.key.keysym.mod & (KMOD_CTRL | KMOD_SHIFT)) == 0
                           && (event.key.keysym.mod & KMOD_ALT) != 0))
                        return 0;
                    break;
                }
            }
            auto image_handle = image_presenter.get_next_image();
            auto *image = image_handle.get();
            if(image->pixel_size != sizeof(std::uint32_t))
                throw std::runtime_error("unsupported pixel_size");
            auto ticks = SDL_GetTicks();
            frame_count++;
            if(ticks - last_fps_report_ticks >= 5000)
            {
                std::cout << frame_count * 1000.0 / (ticks - last_fps_report_ticks) << " FPS"
                          << std::endl;
                frame_count = 0;
                last_fps_report_ticks = ticks;
            }
            auto t = ticks / 32;
            std::uint32_t v = rgb(image, t, t + 0x40, t + 0x80);
            for(std::size_t y = 0; y < image->height; y++)
            {
                for(std::size_t x = 0; x < image->width; x++)
                {
                    auto *pixel = reinterpret_cast<std::uint32_t *>(
                        static_cast<char *>(image->pixels.get()) + x * image->pixel_size
                        + y * image->row_pitch);
                    auto color = v ^ x ^ (((t - x - 64) << 8) ^ (y + t));
                    *pixel = color;
                }
            }
            image_presenter.present_image(image_handle);
        }
    }
    catch(std::runtime_error &e)
    {
        std::cerr << "error: " << e.what() << std::endl;
        return 1;
    }
}
