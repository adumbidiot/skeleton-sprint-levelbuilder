var SksComponents = (function (exports) {
    'use strict';

    function noop() { }
    function assign(tar, src) {
        // @ts-ignore
        for (const k in src)
            tar[k] = src[k];
        return tar;
    }
    function run(fn) {
        return fn();
    }
    function blank_object() {
        return Object.create(null);
    }
    function run_all(fns) {
        fns.forEach(run);
    }
    function is_function(thing) {
        return typeof thing === 'function';
    }
    function safe_not_equal(a, b) {
        return a != a ? b == b : a !== b || ((a && typeof a === 'object') || typeof a === 'function');
    }
    function create_slot(definition, ctx, fn) {
        if (definition) {
            const slot_ctx = get_slot_context(definition, ctx, fn);
            return definition[0](slot_ctx);
        }
    }
    function get_slot_context(definition, ctx, fn) {
        return definition[1]
            ? assign({}, assign(ctx.$$scope.ctx, definition[1](fn ? fn(ctx) : {})))
            : ctx.$$scope.ctx;
    }
    function get_slot_changes(definition, ctx, changed, fn) {
        return definition[1]
            ? assign({}, assign(ctx.$$scope.changed || {}, definition[1](fn ? fn(changed) : {})))
            : ctx.$$scope.changed || {};
    }

    function append(target, node) {
        target.appendChild(node);
    }
    function insert(target, node, anchor) {
        target.insertBefore(node, anchor || null);
    }
    function detach(node) {
        node.parentNode.removeChild(node);
    }
    function element(name) {
        return document.createElement(name);
    }
    function text(data) {
        return document.createTextNode(data);
    }
    function space() {
        return text(' ');
    }
    function listen(node, event, handler, options) {
        node.addEventListener(event, handler, options);
        return () => node.removeEventListener(event, handler, options);
    }
    function attr(node, attribute, value) {
        if (value == null)
            node.removeAttribute(attribute);
        else
            node.setAttribute(attribute, value);
    }
    function children(element) {
        return Array.from(element.childNodes);
    }
    function set_data(text, data) {
        data = '' + data;
        if (text.data !== data)
            text.data = data;
    }
    function toggle_class(element, name, toggle) {
        element.classList[toggle ? 'add' : 'remove'](name);
    }

    let current_component;
    function set_current_component(component) {
        current_component = component;
    }
    // TODO figure out if we still want to support
    // shorthand events, or if we want to implement
    // a real bubbling mechanism
    function bubble(component, event) {
        const callbacks = component.$$.callbacks[event.type];
        if (callbacks) {
            callbacks.slice().forEach(fn => fn(event));
        }
    }

    const dirty_components = [];
    const binding_callbacks = [];
    const render_callbacks = [];
    const flush_callbacks = [];
    const resolved_promise = Promise.resolve();
    let update_scheduled = false;
    function schedule_update() {
        if (!update_scheduled) {
            update_scheduled = true;
            resolved_promise.then(flush);
        }
    }
    function add_render_callback(fn) {
        render_callbacks.push(fn);
    }
    function add_flush_callback(fn) {
        flush_callbacks.push(fn);
    }
    function flush() {
        const seen_callbacks = new Set();
        do {
            // first, call beforeUpdate functions
            // and update components
            while (dirty_components.length) {
                const component = dirty_components.shift();
                set_current_component(component);
                update(component.$$);
            }
            while (binding_callbacks.length)
                binding_callbacks.pop()();
            // then, once components are updated, call
            // afterUpdate functions. This may cause
            // subsequent updates...
            for (let i = 0; i < render_callbacks.length; i += 1) {
                const callback = render_callbacks[i];
                if (!seen_callbacks.has(callback)) {
                    callback();
                    // ...so guard against infinite loops
                    seen_callbacks.add(callback);
                }
            }
            render_callbacks.length = 0;
        } while (dirty_components.length);
        while (flush_callbacks.length) {
            flush_callbacks.pop()();
        }
        update_scheduled = false;
    }
    function update($$) {
        if ($$.fragment) {
            $$.update($$.dirty);
            run_all($$.before_update);
            $$.fragment.p($$.dirty, $$.ctx);
            $$.dirty = null;
            $$.after_update.forEach(add_render_callback);
        }
    }
    const outroing = new Set();
    let outros;
    function transition_in(block, local) {
        if (block && block.i) {
            outroing.delete(block);
            block.i(local);
        }
    }
    function transition_out(block, local, detach, callback) {
        if (block && block.o) {
            if (outroing.has(block))
                return;
            outroing.add(block);
            outros.c.push(() => {
                outroing.delete(block);
                if (callback) {
                    if (detach)
                        block.d(1);
                    callback();
                }
            });
            block.o(local);
        }
    }

    function bind(component, name, callback) {
        if (component.$$.props.indexOf(name) === -1)
            return;
        component.$$.bound[name] = callback;
        callback(component.$$.ctx[name]);
    }
    function mount_component(component, target, anchor) {
        const { fragment, on_mount, on_destroy, after_update } = component.$$;
        fragment.m(target, anchor);
        // onMount happens before the initial afterUpdate
        add_render_callback(() => {
            const new_on_destroy = on_mount.map(run).filter(is_function);
            if (on_destroy) {
                on_destroy.push(...new_on_destroy);
            }
            else {
                // Edge case - component was destroyed immediately,
                // most likely as a result of a binding initialising
                run_all(new_on_destroy);
            }
            component.$$.on_mount = [];
        });
        after_update.forEach(add_render_callback);
    }
    function destroy_component(component, detaching) {
        if (component.$$.fragment) {
            run_all(component.$$.on_destroy);
            component.$$.fragment.d(detaching);
            // TODO null out other refs, including component.$$ (but need to
            // preserve final state?)
            component.$$.on_destroy = component.$$.fragment = null;
            component.$$.ctx = {};
        }
    }
    function make_dirty(component, key) {
        if (!component.$$.dirty) {
            dirty_components.push(component);
            schedule_update();
            component.$$.dirty = blank_object();
        }
        component.$$.dirty[key] = true;
    }
    function init(component, options, instance, create_fragment, not_equal, prop_names) {
        const parent_component = current_component;
        set_current_component(component);
        const props = options.props || {};
        const $$ = component.$$ = {
            fragment: null,
            ctx: null,
            // state
            props: prop_names,
            update: noop,
            not_equal,
            bound: blank_object(),
            // lifecycle
            on_mount: [],
            on_destroy: [],
            before_update: [],
            after_update: [],
            context: new Map(parent_component ? parent_component.$$.context : []),
            // everything else
            callbacks: blank_object(),
            dirty: null
        };
        let ready = false;
        $$.ctx = instance
            ? instance(component, props, (key, value) => {
                if ($$.ctx && not_equal($$.ctx[key], $$.ctx[key] = value)) {
                    if ($$.bound[key])
                        $$.bound[key](value);
                    if (ready)
                        make_dirty(component, key);
                }
            })
            : props;
        $$.update();
        ready = true;
        run_all($$.before_update);
        $$.fragment = create_fragment($$.ctx);
        if (options.target) {
            if (options.hydrate) {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment.l(children(options.target));
            }
            else {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment.c();
            }
            if (options.intro)
                transition_in(component.$$.fragment);
            mount_component(component, options.target, options.anchor);
            flush();
        }
        set_current_component(parent_component);
    }
    class SvelteComponent {
        $destroy() {
            destroy_component(this, 1);
            this.$destroy = noop;
        }
        $on(type, callback) {
            const callbacks = (this.$$.callbacks[type] || (this.$$.callbacks[type] = []));
            callbacks.push(callback);
            return () => {
                const index = callbacks.indexOf(callback);
                if (index !== -1)
                    callbacks.splice(index, 1);
            };
        }
        $set() {
            // overridden by instance, if it has props
        }
    }

    /* svelte\src\Button.html generated by Svelte v3.9.2 */

    function add_css() {
    	var style = element("style");
    	style.id = 'svelte-qjc2uf-style';
    	style.textContent = "button.svelte-qjc2uf{background-color:#ff0000;color:black;text-align:center;text-decoration:none;display:inline-block;vertical-align:middle;padding:.4rem .8rem;font-size:inherit;border:1px solid transparent;border-radius:.25rem;user-select:none;outline:none;transition:all 0.15s ease-in-out;line-height:inherit;cursor:pointer\r\n\t}button.svelte-qjc2uf:active{background-color:#ad0000}";
    	append(document.head, style);
    }

    // (2:1) {#if content}
    function create_if_block(ctx) {
    	var t;

    	return {
    		c() {
    			t = text(ctx.content);
    		},

    		m(target, anchor) {
    			insert(target, t, anchor);
    		},

    		p(changed, ctx) {
    			if (changed.content) {
    				set_data(t, ctx.content);
    			}
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(t);
    			}
    		}
    	};
    }

    function create_fragment(ctx) {
    	var button, t, current, dispose;

    	var if_block = (ctx.content) && create_if_block(ctx);

    	const default_slot_template = ctx.$$slots.default;
    	const default_slot = create_slot(default_slot_template, ctx, null);

    	return {
    		c() {
    			button = element("button");
    			if (if_block) if_block.c();
    			t = space();

    			if (default_slot) default_slot.c();

    			attr(button, "class", "svelte-qjc2uf");
    			dispose = listen(button, "click", ctx.click_handler);
    		},

    		l(nodes) {
    			if (default_slot) default_slot.l(button_nodes);
    		},

    		m(target, anchor) {
    			insert(target, button, anchor);
    			if (if_block) if_block.m(button, null);
    			append(button, t);

    			if (default_slot) {
    				default_slot.m(button, null);
    			}

    			current = true;
    		},

    		p(changed, ctx) {
    			if (ctx.content) {
    				if (if_block) {
    					if_block.p(changed, ctx);
    				} else {
    					if_block = create_if_block(ctx);
    					if_block.c();
    					if_block.m(button, t);
    				}
    			} else if (if_block) {
    				if_block.d(1);
    				if_block = null;
    			}

    			if (default_slot && default_slot.p && changed.$$scope) {
    				default_slot.p(
    					get_slot_changes(default_slot_template, ctx, changed, null),
    					get_slot_context(default_slot_template, ctx, null)
    				);
    			}
    		},

    		i(local) {
    			if (current) return;
    			transition_in(default_slot, local);
    			current = true;
    		},

    		o(local) {
    			transition_out(default_slot, local);
    			current = false;
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(button);
    			}

    			if (if_block) if_block.d();

    			if (default_slot) default_slot.d(detaching);
    			dispose();
    		}
    	};
    }

    function instance($$self, $$props, $$invalidate) {
    	let { content = null } = $$props;

    	let { $$slots = {}, $$scope } = $$props;

    	function click_handler(event) {
    		bubble($$self, event);
    	}

    	$$self.$set = $$props => {
    		if ('content' in $$props) $$invalidate('content', content = $$props.content);
    		if ('$$scope' in $$props) $$invalidate('$$scope', $$scope = $$props.$$scope);
    	};

    	return { content, click_handler, $$slots, $$scope };
    }

    class Button extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-qjc2uf-style")) add_css();
    		init(this, options, instance, create_fragment, safe_not_equal, ["content"]);
    	}
    }

    /* svelte\src\Modal.html generated by Svelte v3.9.2 */

    function add_css$1() {
    	var style = element("style");
    	style.id = 'svelte-10rljv3-style';
    	style.textContent = ".container.svelte-10rljv3{position:fixed;width:50%;height:50%;left:25%;top:25%;background-color:#777777;z-index:999;text-align:center;border-radius:.25rem}.modal.svelte-10rljv3{position:fixed;z-index:999;left:0px;top:0px;width:100%;height:100%;background-color:rgba(0,0,0,0.4);transition:all 0.3s ease-in-out;visibility:hidden;opacity:0}.modal-active.svelte-10rljv3{visibility:visible;opacity:1}";
    	append(document.head, style);
    }

    function create_fragment$1(ctx) {
    	var div1, div0, current, dispose;

    	const default_slot_template = ctx.$$slots.default;
    	const default_slot = create_slot(default_slot_template, ctx, null);

    	return {
    		c() {
    			div1 = element("div");
    			div0 = element("div");

    			if (default_slot) default_slot.c();

    			attr(div0, "class", "container svelte-10rljv3");
    			attr(div1, "class", "modal svelte-10rljv3");
    			toggle_class(div1, "modal-active", ctx.active);

    			dispose = [
    				listen(div0, "click", containerClickHandler),
    				listen(div1, "click", ctx.clickHandler)
    			];
    		},

    		l(nodes) {
    			if (default_slot) default_slot.l(div0_nodes);
    		},

    		m(target, anchor) {
    			insert(target, div1, anchor);
    			append(div1, div0);

    			if (default_slot) {
    				default_slot.m(div0, null);
    			}

    			current = true;
    		},

    		p(changed, ctx) {
    			if (default_slot && default_slot.p && changed.$$scope) {
    				default_slot.p(
    					get_slot_changes(default_slot_template, ctx, changed, null),
    					get_slot_context(default_slot_template, ctx, null)
    				);
    			}

    			if (changed.active) {
    				toggle_class(div1, "modal-active", ctx.active);
    			}
    		},

    		i(local) {
    			if (current) return;
    			transition_in(default_slot, local);
    			current = true;
    		},

    		o(local) {
    			transition_out(default_slot, local);
    			current = false;
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(div1);
    			}

    			if (default_slot) default_slot.d(detaching);
    			run_all(dispose);
    		}
    	};
    }

    function containerClickHandler(e){
    	e.cancelBubble = true;
    }

    function instance$1($$self, $$props, $$invalidate) {
    	let { active } = $$props;
    	
    	function clickHandler(e) {
    		$$invalidate('active', active = !active);
    	}

    	let { $$slots = {}, $$scope } = $$props;

    	$$self.$set = $$props => {
    		if ('active' in $$props) $$invalidate('active', active = $$props.active);
    		if ('$$scope' in $$props) $$invalidate('$$scope', $$scope = $$props.$$scope);
    	};

    	return { active, clickHandler, $$slots, $$scope };
    }

    class Modal extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-10rljv3-style")) add_css$1();
    		init(this, options, instance$1, create_fragment$1, safe_not_equal, ["active"]);
    	}
    }

    /* svelte\src\ImportModal.html generated by Svelte v3.9.2 */

    function add_css$2() {
    	var style = element("style");
    	style.id = 'svelte-1mamwwo-style';
    	style.textContent = ".entry.svelte-1mamwwo{margin:0.25rem 0rem}.title-wrapper.svelte-1mamwwo{text-align:center;background-color:red;margin:0rem 2rem;border-radius:0.25rem;user-select:none}.title.svelte-1mamwwo{text-align:center;text-decoration:none;font-weight:100}.close.svelte-1mamwwo{right:0.5rem;height:1.5rem;font-size:1rem;position:absolute;bottom:1rem}";
    	append(document.head, style);
    }

    // (6:2) <Button on:click={loadLBL}>
    function create_default_slot_4(ctx) {
    	var t;

    	return {
    		c() {
    			t = text("LBL File");
    		},

    		m(target, anchor) {
    			insert(target, t, anchor);
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(t);
    			}
    		}
    	};
    }

    // (9:2) <Button on:click={loadAny}>
    function create_default_slot_3(ctx) {
    	var t;

    	return {
    		c() {
    			t = text("Any File (Guess format)");
    		},

    		m(target, anchor) {
    			insert(target, t, anchor);
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(t);
    			}
    		}
    	};
    }

    // (12:2) <Button on:click={loadAS3}>
    function create_default_slot_2(ctx) {
    	var t;

    	return {
    		c() {
    			t = text("AS3 Array File (Dev)");
    		},

    		m(target, anchor) {
    			insert(target, t, anchor);
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(t);
    			}
    		}
    	};
    }

    // (15:2) <Button on:click="{deactivate}">
    function create_default_slot_1(ctx) {
    	var t;

    	return {
    		c() {
    			t = text("Close");
    		},

    		m(target, anchor) {
    			insert(target, t, anchor);
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(t);
    			}
    		}
    	};
    }

    // (1:0) <Modal bind:active={active}>
    function create_default_slot(ctx) {
    	var div0, t1, div1, t2, div2, t3, div3, t4, div4, current;

    	var button0 = new Button({
    		props: {
    		$$slots: { default: [create_default_slot_4] },
    		$$scope: { ctx }
    	}
    	});
    	button0.$on("click", ctx.loadLBL);

    	var button1 = new Button({
    		props: {
    		$$slots: { default: [create_default_slot_3] },
    		$$scope: { ctx }
    	}
    	});
    	button1.$on("click", ctx.loadAny);

    	var button2 = new Button({
    		props: {
    		$$slots: { default: [create_default_slot_2] },
    		$$scope: { ctx }
    	}
    	});
    	button2.$on("click", ctx.loadAS3);

    	var button3 = new Button({
    		props: {
    		$$slots: { default: [create_default_slot_1] },
    		$$scope: { ctx }
    	}
    	});
    	button3.$on("click", ctx.deactivate);

    	return {
    		c() {
    			div0 = element("div");
    			div0.innerHTML = `<h1 class="title svelte-1mamwwo">Import</h1>`;
    			t1 = space();
    			div1 = element("div");
    			button0.$$.fragment.c();
    			t2 = space();
    			div2 = element("div");
    			button1.$$.fragment.c();
    			t3 = space();
    			div3 = element("div");
    			button2.$$.fragment.c();
    			t4 = space();
    			div4 = element("div");
    			button3.$$.fragment.c();
    			attr(div0, "class", "title-wrapper svelte-1mamwwo");
    			attr(div1, "class", "entry svelte-1mamwwo");
    			attr(div3, "class", "entry svelte-1mamwwo");
    			attr(div4, "class", "close svelte-1mamwwo");
    		},

    		m(target, anchor) {
    			insert(target, div0, anchor);
    			insert(target, t1, anchor);
    			insert(target, div1, anchor);
    			mount_component(button0, div1, null);
    			insert(target, t2, anchor);
    			insert(target, div2, anchor);
    			mount_component(button1, div2, null);
    			insert(target, t3, anchor);
    			insert(target, div3, anchor);
    			mount_component(button2, div3, null);
    			insert(target, t4, anchor);
    			insert(target, div4, anchor);
    			mount_component(button3, div4, null);
    			current = true;
    		},

    		p(changed, ctx) {
    			var button0_changes = {};
    			if (changed.$$scope) button0_changes.$$scope = { changed, ctx };
    			button0.$set(button0_changes);

    			var button1_changes = {};
    			if (changed.$$scope) button1_changes.$$scope = { changed, ctx };
    			button1.$set(button1_changes);

    			var button2_changes = {};
    			if (changed.$$scope) button2_changes.$$scope = { changed, ctx };
    			button2.$set(button2_changes);

    			var button3_changes = {};
    			if (changed.$$scope) button3_changes.$$scope = { changed, ctx };
    			button3.$set(button3_changes);
    		},

    		i(local) {
    			if (current) return;
    			transition_in(button0.$$.fragment, local);

    			transition_in(button1.$$.fragment, local);

    			transition_in(button2.$$.fragment, local);

    			transition_in(button3.$$.fragment, local);

    			current = true;
    		},

    		o(local) {
    			transition_out(button0.$$.fragment, local);
    			transition_out(button1.$$.fragment, local);
    			transition_out(button2.$$.fragment, local);
    			transition_out(button3.$$.fragment, local);
    			current = false;
    		},

    		d(detaching) {
    			if (detaching) {
    				detach(div0);
    				detach(t1);
    				detach(div1);
    			}

    			destroy_component(button0);

    			if (detaching) {
    				detach(t2);
    				detach(div2);
    			}

    			destroy_component(button1);

    			if (detaching) {
    				detach(t3);
    				detach(div3);
    			}

    			destroy_component(button2);

    			if (detaching) {
    				detach(t4);
    				detach(div4);
    			}

    			destroy_component(button3);
    		}
    	};
    }

    function create_fragment$2(ctx) {
    	var updating_active, current;

    	function modal_active_binding(value) {
    		ctx.modal_active_binding.call(null, value);
    		updating_active = true;
    		add_flush_callback(() => updating_active = false);
    	}

    	let modal_props = {
    		$$slots: { default: [create_default_slot] },
    		$$scope: { ctx }
    	};
    	if (ctx.active !== void 0) {
    		modal_props.active = ctx.active;
    	}
    	var modal = new Modal({ props: modal_props });

    	binding_callbacks.push(() => bind(modal, 'active', modal_active_binding));

    	return {
    		c() {
    			modal.$$.fragment.c();
    		},

    		m(target, anchor) {
    			mount_component(modal, target, anchor);
    			current = true;
    		},

    		p(changed, ctx) {
    			var modal_changes = {};
    			if (changed.$$scope) modal_changes.$$scope = { changed, ctx };
    			if (!updating_active && changed.active) {
    				modal_changes.active = ctx.active;
    			}
    			modal.$set(modal_changes);
    		},

    		i(local) {
    			if (current) return;
    			transition_in(modal.$$.fragment, local);

    			current = true;
    		},

    		o(local) {
    			transition_out(modal.$$.fragment, local);
    			current = false;
    		},

    		d(detaching) {
    			destroy_component(modal, detaching);
    		}
    	};
    }

    async function getFilename(){
    	let filename = await window.dialog.showOpenDialog();
    	if(!filename){
    		throw "No Dialog Data";
    	}
    	let data = await readFile(filename[0], 'utf8');
    	return data;
    }

    function readFile(path, encoding){
    	return new Promise((resolve, reject) => {
    		window.fs.readFile(path, encoding, function(err, data){
    			if(err) {
    				reject(err);
    			} else {
    				resolve(data);
    			}
    		});
    	});
    }

    function instance$2($$self, $$props, $$invalidate) {
    	
    	
    	let { active = false } = $$props;
    	function activate(){
    		$$invalidate('active', active = true);
    	}
    	
    	function deactivate(){
    		$$invalidate('active', active = false);
    	}
    	
    	function loadAS3(){
    		getFilename()
    		.then((data) => {
    			window.level.importAS3(data);
    			deactivate();
    		})
    		.catch((e) => {
    			throw e;
    		});
    	}
    	
    	function loadLBL(){
    		getFilename()
    		.then((data) => {
    			window.level.importLBL(data);
    			deactivate();
    		})
    		.catch((e) => {
    			throw e;
    		});
    	}
    	
    	function loadAny(){
    		getFilename()
    		.then((data) => {
    			window.level.import(data);
    			deactivate();
    		})
    		.catch((e) => {
    			throw e;
    		});
    	}

    	function modal_active_binding(value) {
    		active = value;
    		$$invalidate('active', active);
    	}

    	$$self.$set = $$props => {
    		if ('active' in $$props) $$invalidate('active', active = $$props.active);
    	};

    	return {
    		active,
    		activate,
    		deactivate,
    		loadAS3,
    		loadLBL,
    		loadAny,
    		modal_active_binding
    	};
    }

    class ImportModal extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-1mamwwo-style")) add_css$2();
    		init(this, options, instance$2, create_fragment$2, safe_not_equal, ["active", "activate", "deactivate"]);
    	}

    	get activate() {
    		return this.$$.ctx.activate;
    	}

    	get deactivate() {
    		return this.$$.ctx.deactivate;
    	}
    }

    exports.Button = Button;
    exports.ImportModal = ImportModal;

    return exports;

}({}));
