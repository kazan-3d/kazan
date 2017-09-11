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

#ifndef SDL_VIDEO_DRIVER_X11
#error SDL was not built with X11 support
#endif

class Image_presenter;

struct Image
{
    friend class Image_presenter;

private:
    Image(const Image &) = default;
    Image(Image &&) = default;
    Image &operator=(const Image &) = default;
    Image &operator=(Image &&) = default;
    Image(void *pixels,
          std::size_t row_pitch,
          std::size_t width,
          std::size_t height,
          std::size_t pixel_size) noexcept : pixels(pixels),
                                             row_pitch(row_pitch),
                                             width(width),
                                             height(height),
                                             pixel_size(pixel_size)
    {
    }

public:
    virtual ~Image()
    {
    }
    void *pixels;
    std::size_t row_pitch;
    std::size_t width;
    std::size_t height;
    std::size_t pixel_size;
};

class Image_presenter
{
    Image_presenter(const Image_presenter &) = delete;
    Image_presenter &operator=(const Image_presenter &) = delete;

private:
    struct Presentable_image : public Image
    {
    };

private:
    xcb_connection_t *connection;
    xcb_window_t window;

private:
    static xcb_query_extension_cookie_t query_extension(xcb_connection_t *connection,
                                                        const char *extension_name) noexcept
    {
        return xcb_query_extension(connection, std::strlen(extension_name), extension_name);
    }
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

public:
    Image_presenter(xcb_connection_t *connection, xcb_window_t window)
        : connection(connection), window(window)
    {
        auto present_cookie = query_extension(connection, "Present");
        auto mit_shm_cookie = query_extension(connection, "MIT-SHM");
        auto present_reply =
            Query_extension_reply(xcb_query_extension_reply(connection, present_cookie, nullptr));
        auto mit_shm_reply =
            Query_extension_reply(xcb_query_extension_reply(connection, mit_shm_cookie, nullptr));
        if(!present_reply || !present_reply->present)
            throw std::runtime_error("X server doesn't support Present extension");
        if(!mit_shm_reply || !mit_shm_reply->present)
            throw std::runtime_error("X server doesn't support MIT-SHM extension");
        throw std::runtime_error("Image_presenter::Image_presenter is not implemented");
#warning finish implementing Image_presenter::Image_presenter
    }
    std::unique_ptr<Image> get_next_image()
    {
#if 1
        throw std::runtime_error("Image_presenter::get_next_image is not implemented");
#warning finish implementing Image_presenter::get_next_image
#else
        auto presentable_image = std::make_unique<Presentable_image>();
        return std::unique_ptr<Image>(static_cast<Image *>(presentable_image.release()));
#endif
    }
    void present_image(std::unique_ptr<Image> image)
    {
        assert(dynamic_cast<Presentable_image *>(image.get()));
        auto presentable_image =
            std::unique_ptr<Presentable_image>(static_cast<Presentable_image *>(image.release()));
        throw std::runtime_error("Image_presenter::present_image is not implemented");
#warning finish implementing Image_presenter::present_image
    }
};

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
                                    640,
                                    480,
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
        Image_presenter image_presenter(XGetXCBConnection(wm_info.info.x11.display),
                                        static_cast<xcb_window_t>(wm_info.info.x11.window));
        SDL_Event event;
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
            auto image = image_presenter.get_next_image();
            if(image->pixel_size != sizeof(std::uint32_t))
                throw std::runtime_error("unsupported pixel_size");
            for(std::size_t y = 0; y < image->height; y++)
            {
                for(std::size_t x = 0; x < image->width; x++)
                {
                    *reinterpret_cast<std::uint32_t *>(static_cast<char *>(image->pixels)
                                                       + x * image->pixel_size
                                                       + y * image->row_pitch) = 0x123456UL;
                }
            }
            image_presenter.present_image(std::move(image));
        }
    }
    catch(std::runtime_error &e)
    {
        std::cerr << "error: " << e.what() << std::endl;
        return 1;
    }
}
