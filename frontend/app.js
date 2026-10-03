import { request, errorMessage, ApiError } from './api.js';
import { demoHotels, images, imageUrl } from './data.js';

const $ = (selector) => document.querySelector(selector);
const escape = (value) => String(value ?? '').replace(/[&<>"']/g, (character) => ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[character]));
const icon = (name) => `<i data-lucide="${name}"></i>`;
const icons = () => window.lucide?.createIcons();
const storage = {
  read(store, key, fallback) { try { return store.getItem(key) || fallback; } catch { return fallback; } },
  write(store, key, value) { try { value === null ? store.removeItem(key) : store.setItem(key, value); } catch { /* Storage can be unavailable in private browser contexts. */ } },
};
let savedFavorites;
try { savedFavorites = JSON.parse(storage.read(localStorage, 'mesto.favorites', '[]')); } catch { savedFavorites = []; }
const state = {
  hotels:[], demo:false, loaded:false, hasMore:false, loading:false, retryAppend:false, query:'', city:'all', favoritesOnly:false,
  favorites:new Set(Array.isArray(savedFavorites) ? savedFavorites.filter((id) => typeof id === 'string') : []),
  token:storage.read(sessionStorage, 'mesto.token', ''), user:null, authMode:'login', editing:null,
  detail:null, detailController:null, deleting:null,
};
let toastTimer;

function notify(message) {
  clearTimeout(toastTimer);
  $('#toast').textContent = message;
  $('#toast').hidden = false;
  toastTimer = setTimeout(() => { $('#toast').hidden = true; }, 3500);
}

function photo(hotel) {
  let hash = 0;
  for (const character of hotel.id) hash = (hash + character.charCodeAt(0)) % images.length;
  return hotel.photo || images[hash];
}

function updateFavorites() {
  $('#favorites-count').textContent = [...state.favorites].filter((id) => state.hotels.some((hotel) => hotel.id === id)).length;
  $('#favorites-filter').setAttribute('aria-pressed', state.favoritesOnly);
  $('#favorites-filter').title = state.favoritesOnly ? 'Все отели' : 'Только избранное';
  $('#favorites-filter').setAttribute('aria-label', $('#favorites-filter').title);
  document.querySelectorAll('[data-favorite]').forEach((button) => {
    const active = state.favorites.has(button.dataset.favorite);
    button.setAttribute('aria-pressed', active);
    button.title = active ? 'Убрать из избранного' : 'В избранное';
    button.setAttribute('aria-label', button.title);
    const text = button.querySelector('span');
    if (text) text.textContent = active ? 'В избранном' : 'В избранное';
  });
}

function toggleFavorite(id) {
  const active = state.favorites.has(id);
  active ? state.favorites.delete(id) : state.favorites.add(id);
  storage.write(localStorage, 'mesto.favorites', JSON.stringify([...state.favorites]));
  if (state.favoritesOnly) renderCatalog();
  else updateFavorites();
  notify(active ? 'Удалено из избранного' : 'Добавлено в избранное');
}

function renderCities() {
  const cities = [...new Set(state.hotels.map((hotel) => hotel.city))].sort((a, b) => a.localeCompare(b, 'ru'));
  if (!cities.includes(state.city)) state.city = 'all';
  $('#city-filters').innerHTML = `<button type="button" data-city="all" aria-pressed="${state.city === 'all'}">Все места</button>`
    + cities.slice(0, 5).map((city) => `<button type="button" data-city="${escape(city)}" aria-pressed="${state.city === city}">${escape(city)}</button>`).join('');
}

function renderCatalog() {
  renderCities();
  const query = state.query.toLocaleLowerCase('ru');
  const hotels = state.hotels.filter((hotel) => (state.city === 'all' || hotel.city === state.city)
    && (!state.favoritesOnly || state.favorites.has(hotel.id))
    && `${hotel.name} ${hotel.city}`.toLocaleLowerCase('ru').includes(query));
  const order = $('#sort-select').value;
  if (order !== 'recommended') hotels.sort((a, b) => a[order].localeCompare(b[order], 'ru'));
  $('#collection-label').textContent = state.demo ? 'Коллекция мест · демо-подборка' : 'Коллекция мест';
  $('#catalog-title').textContent = state.favoritesOnly ? 'Ваши любимые места' : 'Отдых начинается здесь';
  $('#result-count').textContent = state.loaded ? `${hotels.length} ${hotelWord(hotels.length)}` : '';
  $('#hotel-grid').setAttribute('aria-busy', state.loading);
  $('#catalog-status').innerHTML = '';
  if (!hotels.length && state.loaded) {
    $('#catalog-status').innerHTML = `<h3>${state.favoritesOnly ? 'Сохраните свое первое место' : 'Пока ничего не нашлось'}</h3>
      <p>${state.favoritesOnly ? 'Здесь будут места, к которым хочется вернуться.' : 'Попробуйте другой город или название.'}</p>
      <button id="reset-filters" class="button button-outline">Все места ${icon('arrow-right')}</button>`;
  }
  $('#hotel-grid').innerHTML = hotels.map((hotel) => {
    const url = `#hotel/${encodeURIComponent(hotel.id)}`;
    return `<article class="hotel-card">
      <div class="hotel-image-wrap"><a href="${url}" aria-label="Открыть ${escape(hotel.name)}"><img class="hotel-image" src="${imageUrl(photo(hotel))}" alt="Иллюстрация гостиничного отдыха" loading="lazy" width="900" height="634"></a>
        <button class="icon-button hotel-favorite" data-favorite="${escape(hotel.id)}" title="В избранное" aria-label="В избранное" aria-pressed="false">${icon('heart')}</button>
        <span class="photo-badge">${state.demo ? 'Демо-отель' : 'Иллюстрация'}</span>
      </div>
      <div class="hotel-meta">${icon('map-pin')} ${escape(hotel.city)}</div><h3><a href="${url}">${escape(hotel.name)}</a></h3>
      <p>${escape(hotel.description || hotel.address)}</p>
      <div class="hotel-card-bottom"><span class="hotel-style">${escape(hotel.style || 'Место для отдыха')}</span><a href="${url}">Об отеле ${icon('arrow-up-right')}</a></div>
    </article>`;
  }).join('');
  $('#load-more').hidden = !state.hasMore;
  $('#load-more').disabled = state.loading;
  updateFavorites();
  icons();
}

function hotelWord(count) {
  if (count % 100 >= 11 && count % 100 <= 14) return 'отелей';
  return count % 10 === 1 ? 'отель' : count % 10 >= 2 && count % 10 <= 4 ? 'отеля' : 'отелей';
}

async function loadHotels(append = false) {
  if (state.loading) return;
  state.loading = true;
  if (!append) {
    $('#hotel-grid').innerHTML = Array.from({ length:3 }, () => '<div class="skeleton" aria-hidden="true"></div>').join('');
    $('#catalog-status').innerHTML = '';
  }
  $('#hotel-grid').setAttribute('aria-busy', 'true');
  $('#load-more').disabled = true;
  try {
    const offset = append ? state.hotels.length : 0;
    const hotels = await request(`/hotels?limit=100&offset=${offset}`);
    if (!Array.isArray(hotels)) throw new ApiError('Сервер вернул некорректный список отелей.');
    state.demo = !append && hotels.length === 0;
    state.hotels = state.demo ? demoHotels : append ? [...state.hotels, ...hotels] : hotels;
    state.hasMore = !state.demo && hotels.length === 100;
    state.loaded = true;
    renderCatalog();
  } catch (error) {
    state.retryAppend = append;
    if (!append) $('#hotel-grid').innerHTML = '';
    $('#catalog-status').innerHTML = `<h3>Не получилось загрузить отели</h3><p>${escape(errorMessage(error))}</p><button id="retry-catalog" class="button button-outline">Попробовать снова ${icon('refresh-cw')}</button>`;
    $('#result-count').textContent = '';
    $('#load-more').hidden = true;
    icons();
  } finally {
    state.loading = false;
    $('#hotel-grid').setAttribute('aria-busy', 'false');
    $('#load-more').disabled = false;
  }
}

function resetFilters() {
  state.query = ''; state.city = 'all'; state.favoritesOnly = false;
  $('#search-input').value = '';
  if (state.loaded) renderCatalog();
}

async function route() {
  state.detailController?.abort();
  const match = location.hash.match(/^#hotel\/([^/]+)$/);
  $('#catalog-page').hidden = Boolean(match);
  $('#detail-page').hidden = !match;
  if (!match) {
    document.title = 'Место — отели с характером';
    if (location.hash === '#favorites') {
      state.favoritesOnly = true; state.city = 'all'; state.query = ''; $('#search-input').value = '';
      if (state.loaded) renderCatalog();
      $('#catalog').scrollIntoView({ behavior:'smooth' });
    }
    return;
  }
  state.detail = null;
  window.scrollTo({ top:0, behavior:'instant' });
  $('#detail-page').innerHTML = '<div class="detail-loading"><p class="text-muted">Находим ваше место…</p></div>';
  let id;
  try { id = decodeURIComponent(match[1]); } catch { renderDetailError('Не удалось найти этот отель.'); return; }
  const controller = new AbortController();
  state.detailController = controller;
  try {
    const demo = demoHotels.find((hotel) => hotel.id === id);
    const hotel = demo || await request(`/hotels/${encodeURIComponent(id)}`, { signal:controller.signal });
    if (controller.signal.aborted) return;
    state.detail = { ...hotel, demo:Boolean(demo) };
    renderDetail();
  } catch (error) {
    if (error.name !== 'AbortError') renderDetailError(error.status === 404 ? 'Это место не найдено.' : errorMessage(error));
  }
}

function renderDetailError(message) {
  $('#detail-page').innerHTML = `<div class="catalog-status section-width"><h2>${escape(message)}</h2><p></p><a href="#catalog" class="button button-outline">${icon('arrow-left')} К коллекции</a></div>`;
  icons();
}

function renderDetail() {
  const hotel = state.detail;
  if (!hotel || $('#detail-page').hidden) return;
  document.title = `${hotel.name} — Место`;
  $('#detail-page').innerHTML = `<div class="detail-top section-width"><a class="back-link" href="#catalog">${icon('arrow-left')} К коллекции</a>
    ${state.user?.role === 'admin' && !hotel.demo ? `<div class="detail-admin"><button class="icon-button" data-edit="${escape(hotel.id)}" title="Редактировать отель" aria-label="Редактировать отель">${icon('pencil')}</button><button class="icon-button" data-delete="${escape(hotel.id)}" title="Удалить отель" aria-label="Удалить отель">${icon('trash-2')}</button></div>` : ''}</div>
    <div class="detail-photo"><img src="${imageUrl(photo(hotel), 1800)}" alt="Иллюстрация гостиничного отдыха"><span class="photo-badge">${hotel.demo ? 'Демо-отель · вымышленное место' : 'Фото для оформления'}</span></div>
    <div class="detail-content section-width"><div><p class="eyebrow text-muted">${escape(hotel.city)}</p><h1 tabindex="-1">${escape(hotel.name)}</h1>
      <p class="detail-description">${escape(hotel.description || 'Описание этого места пока не добавлено.')}</p><div class="detail-address">${icon('map-pin')}<span>${escape(hotel.city)}<br>${escape(hotel.address)}</span></div></div>
      <aside class="detail-side"><p class="eyebrow">Ваше следующее место</p><p>Некоторые места остаются с нами даже после окончания поездки.</p>
        <button class="button button-dark" data-favorite="${escape(hotel.id)}" aria-pressed="false">${icon('heart')}<span>В избранное</span></button></aside></div>`;
  icons(); updateFavorites();
}

function updateAccount() {
  const user = state.user;
  $('#account-button span').textContent = user ? user.first_name : 'Войти';
  $('#account-button').title = user ? 'Открыть профиль' : 'Войти';
  $('#add-hotel').hidden = user?.role !== 'admin';
  if (user) {
    $('#account-title').textContent = `${user.first_name} ${user.last_name}`;
    $('#account-info').innerHTML = `<dt>Email</dt><dd>${escape(user.email)}</dd><dt>Роль</dt><dd>${escape({ customer:'Гость', manager:'Менеджер', admin:'Администратор' }[user.role] || user.role)}</dd>`;
  }
  renderDetail();
}

function clearSession() {
  state.user = null; state.token = '';
  storage.write(sessionStorage, 'mesto.token', null);
  updateAccount();
}

async function restoreSession() {
  if (!state.token) return;
  const token = state.token;
  try {
    const user = await request('/auth/me', { token });
    if (state.token === token) { state.user = user; updateAccount(); }
  } catch (error) {
    if (state.token !== token) return;
    if (error.status === 401) clearSession();
    else notify('Не удалось восстановить профиль. Попробуйте войти снова.');
  }
}

function authMode(mode) {
  state.authMode = mode;
  const register = mode === 'register';
  $('#auth-title').textContent = register ? 'Давайте знакомиться.' : 'С возвращением.';
  $('#registration-fields').hidden = !register;
  $('#registration-fields').querySelectorAll('input').forEach((input) => { input.required = register; input.disabled = !register; });
  $('#password-note').hidden = !register;
  $('#auth-form [name="password"]').autocomplete = register ? 'new-password' : 'current-password';
  $('#auth-submit').innerHTML = `${register ? 'Создать аккаунт' : 'Войти'} ${icon('arrow-right')}`;
  $('#auth-error').textContent = '';
  $('#auth-tabs').querySelectorAll('button').forEach((button) => button.setAttribute('aria-pressed', button.dataset.authMode === mode));
  icons();
}

function openAuth() {
  $('#auth-form').reset();
  $('#auth-form [name="password"]').type = 'password';
  $('#password-toggle').innerHTML = icon('eye');
  $('#password-toggle').title = 'Показать пароль';
  $('#password-toggle').setAttribute('aria-label', 'Показать пароль');
  authMode('login');
  $('#auth-dialog').showModal();
}

async function authenticated(path, options) {
  try { return await request(path, { ...options, token:state.token }); }
  catch (error) {
    if (error.status === 401) { clearSession(); notify('Сессия истекла. Войдите снова.'); }
    throw error;
  }
}

function openHotelForm(hotel = null) {
  state.editing = hotel;
  $('#hotel-form').reset();
  $('#hotel-error').textContent = '';
  $('#hotel-form-title').textContent = hotel ? 'Редактировать отель' : 'Новое место';
  if (hotel) for (const field of ['name', 'city', 'address', 'description']) $('#hotel-form').elements[field].value = hotel[field] || '';
  $('#hotel-dialog').showModal();
}

$('#year').textContent = new Date().getFullYear();
document.addEventListener('click', (event) => {
  const button = event.target.closest('button');
  if (!button) return;
  if (button.dataset.close) $(`#${button.dataset.close}`).close();
  if (button.dataset.favorite) toggleFavorite(button.dataset.favorite);
  if (button.dataset.authMode) authMode(button.dataset.authMode);
  if (button.dataset.city) { state.city = button.dataset.city; renderCatalog(); }
  if (button.id === 'reset-filters') resetFilters();
  if (button.id === 'retry-catalog') loadHotels(state.retryAppend);
  if (button.dataset.edit && state.user?.role === 'admin') openHotelForm(state.detail);
  if (button.dataset.delete && state.user?.role === 'admin') {
    state.deleting = state.detail;
    $('#delete-name').textContent = state.detail.name;
    $('#delete-error').textContent = '';
    $('#delete-dialog').showModal();
  }
});
document.querySelectorAll('dialog').forEach((dialog) => dialog.addEventListener('click', (event) => {
  const rect = dialog.getBoundingClientRect();
  if (event.target === dialog && (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom)) dialog.close();
}));
$('#search-form').addEventListener('submit', (event) => {
  event.preventDefault(); state.query = $('#search-input').value.trim();
  if (state.loaded) renderCatalog();
  $('#catalog').scrollIntoView({ behavior:'smooth' });
});
$('#search-input').addEventListener('input', () => { state.query = $('#search-input').value.trim(); if (state.loaded) renderCatalog(); });
$('#sort-select').addEventListener('change', () => { if (state.loaded) renderCatalog(); });
$('#favorites-filter').addEventListener('click', () => { state.favoritesOnly = !state.favoritesOnly; renderCatalog(); });
$('#nav-favorites').addEventListener('click', () => {
  if (location.hash === '#favorites') { state.favoritesOnly = true; renderCatalog(); $('#catalog').scrollIntoView({ behavior:'smooth' }); }
  else location.hash = 'favorites';
});
document.querySelectorAll('.nav-catalog,.hero-link,.brand').forEach((link) => link.addEventListener('click', resetFilters));
$('#load-more').addEventListener('click', () => loadHotels(true));
$('#account-button').addEventListener('click', () => state.user ? $('#account-dialog').showModal() : openAuth());
$('#logout-button').addEventListener('click', () => { clearSession(); $('#account-dialog').close(); notify('Вы вышли из аккаунта'); });
$('#password-toggle').addEventListener('click', () => {
  const password = $('#auth-form [name="password"]');
  const visible = password.type === 'password';
  password.type = visible ? 'text' : 'password';
  $('#password-toggle').innerHTML = icon(visible ? 'eye-off' : 'eye');
  $('#password-toggle').title = visible ? 'Скрыть пароль' : 'Показать пароль';
  $('#password-toggle').setAttribute('aria-label', $('#password-toggle').title); icons();
});
$('#auth-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const data = Object.fromEntries(new FormData(form));
  const register = state.authMode === 'register';
  $('#auth-error').textContent = '';
  if (register && ([...data.password].length < 12 || [...data.password].length > 128)) {
    $('#auth-error').textContent = 'Пароль должен содержать от 12 до 128 символов.'; return;
  }
  form.querySelectorAll('button').forEach((button) => { button.disabled = true; });
  $('#auth-tabs').querySelectorAll('button').forEach((button) => { button.disabled = true; });
  try {
    const result = await request(`/auth/${register ? 'register' : 'login'}`, { method:'POST', body:data });
    state.token = result.access_token; state.user = result.user;
    storage.write(sessionStorage, 'mesto.token', state.token);
    updateAccount(); $('#auth-dialog').close(); form.reset();
    notify(register ? 'Добро пожаловать в Место' : `Рады видеть вас, ${state.user.first_name}`);
  } catch (error) { $('#auth-error').textContent = error.status === 401 ? 'Неверный email или пароль.' : errorMessage(error); }
  finally {
    form.querySelectorAll('button').forEach((button) => { button.disabled = false; });
    $('#auth-tabs').querySelectorAll('button').forEach((button) => { button.disabled = false; });
  }
});
$('#add-hotel').addEventListener('click', () => openHotelForm());
$('#hotel-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const form = event.currentTarget;
  const data = Object.fromEntries(new FormData(form));
  for (const field of ['name', 'city', 'address']) data[field] = data[field].trim();
  data.description = data.description.trim() || null;
  $('#hotel-error').textContent = '';
  const submit = form.querySelector('[type="submit"]'); submit.disabled = true;
  const editing = state.editing;
  try {
    const hotel = await authenticated(editing ? `/hotels/${encodeURIComponent(editing.id)}` : '/hotels', { method:editing ? 'PUT' : 'POST', body:data });
    $('#hotel-dialog').close(); notify(editing ? 'Изменения сохранены' : 'Новое место добавлено');
    await loadHotels();
    state.detail = { ...hotel, demo:false };
    if (location.hash === `#hotel/${hotel.id}`) renderDetail();
    else location.hash = `hotel/${hotel.id}`;
  } catch (error) { $('#hotel-error').textContent = errorMessage(error); }
  finally { submit.disabled = false; }
});
$('#confirm-delete').addEventListener('click', async () => {
  if (!state.deleting) return;
  const button = $('#confirm-delete'); button.disabled = true;
  const hotel = state.deleting;
  try {
    await authenticated(`/hotels/${encodeURIComponent(hotel.id)}`, { method:'DELETE' });
    state.favorites.delete(hotel.id);
    storage.write(localStorage, 'mesto.favorites', JSON.stringify([...state.favorites]));
    $('#delete-dialog').close(); state.detail = null;
    location.hash = 'catalog'; await loadHotels(); notify('Отель удален');
  } catch (error) { $('#delete-error').textContent = errorMessage(error); }
  finally { button.disabled = false; }
});
window.addEventListener('hashchange', route);

icons(); authMode('login'); updateAccount();
loadHotels(); restoreSession(); route();
