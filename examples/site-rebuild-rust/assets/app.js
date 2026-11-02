// Kinora: the few things the page does on its own. Everything works without this file too.
(() => {
  'use strict';

  // Phone menu.
  const drawer = document.getElementById('drawer');
  const openBtn = document.querySelector('.menu-btn');
  const closeBtn = document.querySelector('.menu-close');
  const setOpen = open => {
    if (!drawer || !openBtn) return;
    drawer.hidden = !open;
    openBtn.setAttribute('aria-expanded', String(open));
    document.body.style.overflow = open ? 'hidden' : '';
    (open ? closeBtn : openBtn)?.focus();
  };
  openBtn?.addEventListener('click', () => setOpen(true));
  closeBtn?.addEventListener('click', () => setOpen(false));
  document.addEventListener('keydown', e => { if (e.key === 'Escape' && drawer && !drawer.hidden) setOpen(false); });

  // Carousel arrows: scroll by one card, hide an arrow at its end.
  for (const car of document.querySelectorAll('.carousel')) {
    const track = car.querySelector('.car-track');
    const prev = car.querySelector('.prev');
    const next = car.querySelector('.next');
    if (!track || !prev || !next) continue;
    const step = () => (track.firstElementChild?.getBoundingClientRect().width || 300) + 20;
    const sync = () => {
      prev.disabled = track.scrollLeft < 4;
      next.disabled = track.scrollLeft + track.clientWidth >= track.scrollWidth - 4;
    };
    prev.addEventListener('click', () => track.scrollBy({ left: -step(), behavior: 'smooth' }));
    next.addEventListener('click', () => track.scrollBy({ left: step(), behavior: 'smooth' }));
    track.addEventListener('scroll', sync, { passive: true });
    window.addEventListener('resize', sync);
    sync();
  }

  // Likes on community remixes (kept in this page only).
  for (const btn of document.querySelectorAll('.like')) {
    btn.addEventListener('click', () => {
      const on = btn.getAttribute('aria-pressed') !== 'true';
      btn.setAttribute('aria-pressed', String(on));
      const n = btn.querySelector('span');
      if (n) n.textContent = String((parseInt(n.textContent, 10) || 0) + (on ? 1 : -1));
    });
  }
})();
