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
    function create_slot(definition, ctx, $$scope, fn) {
        if (definition) {
            const slot_ctx = get_slot_context(definition, ctx, $$scope, fn);
            return definition[0](slot_ctx);
        }
    }
    function get_slot_context(definition, ctx, $$scope, fn) {
        return definition[1] && fn
            ? assign($$scope.ctx.slice(), definition[1](fn(ctx)))
            : $$scope.ctx;
    }
    function get_slot_changes(definition, $$scope, dirty, fn) {
        if (definition[2] && fn) {
            const lets = definition[2](fn(dirty));
            if (typeof $$scope.dirty === 'object') {
                const merged = [];
                const len = Math.max($$scope.dirty.length, lets.length);
                for (let i = 0; i < len; i += 1) {
                    merged[i] = $$scope.dirty[i] | lets[i];
                }
                return merged;
            }
            return $$scope.dirty | lets;
        }
        return $$scope.dirty;
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
        else if (node.getAttribute(attribute) !== value)
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
        if ($$.fragment !== null) {
            $$.update();
            run_all($$.before_update);
            const dirty = $$.dirty;
            $$.dirty = [-1];
            $$.fragment && $$.fragment.p($$.ctx, dirty);
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
        const index = component.$$.props[name];
        if (index !== undefined) {
            component.$$.bound[index] = callback;
            callback(component.$$.ctx[index]);
        }
    }
    function create_component(block) {
        block && block.c();
    }
    function mount_component(component, target, anchor) {
        const { fragment, on_mount, on_destroy, after_update } = component.$$;
        fragment && fragment.m(target, anchor);
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
        const $$ = component.$$;
        if ($$.fragment !== null) {
            run_all($$.on_destroy);
            $$.fragment && $$.fragment.d(detaching);
            // TODO null out other refs, including component.$$ (but need to
            // preserve final state?)
            $$.on_destroy = $$.fragment = null;
            $$.ctx = [];
        }
    }
    function make_dirty(component, i) {
        if (component.$$.dirty[0] === -1) {
            dirty_components.push(component);
            schedule_update();
            component.$$.dirty.fill(0);
        }
        component.$$.dirty[(i / 31) | 0] |= (1 << (i % 31));
    }
    function init(component, options, instance, create_fragment, not_equal, props, dirty = [-1]) {
        const parent_component = current_component;
        set_current_component(component);
        const prop_values = options.props || {};
        const $$ = component.$$ = {
            fragment: null,
            ctx: null,
            // state
            props,
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
            dirty
        };
        let ready = false;
        $$.ctx = instance
            ? instance(component, prop_values, (i, ret, ...rest) => {
                const value = rest.length ? rest[0] : ret;
                if ($$.ctx && not_equal($$.ctx[i], $$.ctx[i] = value)) {
                    if ($$.bound[i])
                        $$.bound[i](value);
                    if (ready)
                        make_dirty(component, i);
                }
                return ret;
            })
            : [];
        $$.update();
        ready = true;
        run_all($$.before_update);
        // `false` as a special case of no DOM component
        $$.fragment = create_fragment ? create_fragment($$.ctx) : false;
        if (options.target) {
            if (options.hydrate) {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment && $$.fragment.l(children(options.target));
            }
            else {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment && $$.fragment.c();
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

    /* svelte\src\Button.html generated by Svelte v3.17.3 */

    function add_css() {
    	var style = element("style");
    	style.id = "svelte-qjc2uf-style";
    	style.textContent = "button.svelte-qjc2uf{background-color:#ff0000;color:black;text-align:center;text-decoration:none;display:inline-block;vertical-align:middle;padding:.4rem .8rem;font-size:inherit;border:1px solid transparent;border-radius:.25rem;user-select:none;outline:none;transition:all 0.15s ease-in-out;line-height:inherit;cursor:pointer\r\n\t}button.svelte-qjc2uf:active{background-color:#ad0000}";
    	append(document.head, style);
    }

    // (2:1) {#if content}
    function create_if_block(ctx) {
    	let t;

    	return {
    		c() {
    			t = text(/*content*/ ctx[0]);
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		p(ctx, dirty) {
    			if (dirty & /*content*/ 1) set_data(t, /*content*/ ctx[0]);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    function create_fragment(ctx) {
    	let button;
    	let t;
    	let current;
    	let dispose;
    	let if_block = /*content*/ ctx[0] && create_if_block(ctx);
    	const default_slot_template = /*$$slots*/ ctx[2].default;
    	const default_slot = create_slot(default_slot_template, ctx, /*$$scope*/ ctx[1], null);

    	return {
    		c() {
    			button = element("button");
    			if (if_block) if_block.c();
    			t = space();
    			if (default_slot) default_slot.c();
    			attr(button, "class", "svelte-qjc2uf");
    		},
    		m(target, anchor) {
    			insert(target, button, anchor);
    			if (if_block) if_block.m(button, null);
    			append(button, t);

    			if (default_slot) {
    				default_slot.m(button, null);
    			}

    			current = true;
    			dispose = listen(button, "click", /*click_handler*/ ctx[3]);
    		},
    		p(ctx, [dirty]) {
    			if (/*content*/ ctx[0]) {
    				if (if_block) {
    					if_block.p(ctx, dirty);
    				} else {
    					if_block = create_if_block(ctx);
    					if_block.c();
    					if_block.m(button, t);
    				}
    			} else if (if_block) {
    				if_block.d(1);
    				if_block = null;
    			}

    			if (default_slot && default_slot.p && dirty & /*$$scope*/ 2) {
    				default_slot.p(get_slot_context(default_slot_template, ctx, /*$$scope*/ ctx[1], null), get_slot_changes(default_slot_template, /*$$scope*/ ctx[1], dirty, null));
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
    			if (detaching) detach(button);
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
    		if ("content" in $$props) $$invalidate(0, content = $$props.content);
    		if ("$$scope" in $$props) $$invalidate(1, $$scope = $$props.$$scope);
    	};

    	return [content, $$scope, $$slots, click_handler];
    }

    class Button extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-qjc2uf-style")) add_css();
    		init(this, options, instance, create_fragment, safe_not_equal, { content: 0 });
    	}
    }

    /* svelte\src\Modal.html generated by Svelte v3.17.3 */

    function add_css$1() {
    	var style = element("style");
    	style.id = "svelte-10rljv3-style";
    	style.textContent = ".container.svelte-10rljv3{position:fixed;width:50%;height:50%;left:25%;top:25%;background-color:#777777;z-index:999;text-align:center;border-radius:.25rem}.modal.svelte-10rljv3{position:fixed;z-index:999;left:0px;top:0px;width:100%;height:100%;background-color:rgba(0,0,0,0.4);transition:all 0.3s ease-in-out;visibility:hidden;opacity:0}.modal-active.svelte-10rljv3{visibility:visible;opacity:1}";
    	append(document.head, style);
    }

    function create_fragment$1(ctx) {
    	let div1;
    	let div0;
    	let current;
    	let dispose;
    	const default_slot_template = /*$$slots*/ ctx[3].default;
    	const default_slot = create_slot(default_slot_template, ctx, /*$$scope*/ ctx[2], null);

    	return {
    		c() {
    			div1 = element("div");
    			div0 = element("div");
    			if (default_slot) default_slot.c();
    			attr(div0, "class", "container svelte-10rljv3");
    			attr(div1, "class", "modal svelte-10rljv3");
    			toggle_class(div1, "modal-active", /*active*/ ctx[0]);
    		},
    		m(target, anchor) {
    			insert(target, div1, anchor);
    			append(div1, div0);

    			if (default_slot) {
    				default_slot.m(div0, null);
    			}

    			current = true;

    			dispose = [
    				listen(div0, "click", cancelHandler),
    				listen(div0, "mousedown", cancelHandler),
    				listen(div1, "click", /*clickHandler*/ ctx[1])
    			];
    		},
    		p(ctx, [dirty]) {
    			if (default_slot && default_slot.p && dirty & /*$$scope*/ 4) {
    				default_slot.p(get_slot_context(default_slot_template, ctx, /*$$scope*/ ctx[2], null), get_slot_changes(default_slot_template, /*$$scope*/ ctx[2], dirty, null));
    			}

    			if (dirty & /*active*/ 1) {
    				toggle_class(div1, "modal-active", /*active*/ ctx[0]);
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
    			if (detaching) detach(div1);
    			if (default_slot) default_slot.d(detaching);
    			run_all(dispose);
    		}
    	};
    }

    function cancelHandler(e) {
    	e.cancelBubble = true;
    }

    function instance$1($$self, $$props, $$invalidate) {
    	let { active } = $$props;

    	function clickHandler(e) {
    		$$invalidate(0, active = !active);
    	}

    	let { $$slots = {}, $$scope } = $$props;

    	$$self.$set = $$props => {
    		if ("active" in $$props) $$invalidate(0, active = $$props.active);
    		if ("$$scope" in $$props) $$invalidate(2, $$scope = $$props.$$scope);
    	};

    	return [active, clickHandler, $$scope, $$slots];
    }

    class Modal extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-10rljv3-style")) add_css$1();
    		init(this, options, instance$1, create_fragment$1, safe_not_equal, { active: 0 });
    	}
    }

    /* svelte\src\ImportModal.html generated by Svelte v3.17.3 */

    function add_css$2() {
    	var style = element("style");
    	style.id = "svelte-1mamwwo-style";
    	style.textContent = ".entry.svelte-1mamwwo{margin:0.25rem 0rem}.title-wrapper.svelte-1mamwwo{text-align:center;background-color:red;margin:0rem 2rem;border-radius:0.25rem;user-select:none}.title.svelte-1mamwwo{text-align:center;text-decoration:none;font-weight:100}.close.svelte-1mamwwo{right:0.5rem;height:1.5rem;font-size:1rem;position:absolute;bottom:1rem}";
    	append(document.head, style);
    }

    // (6:2) <Button on:click={loadLBL}>
    function create_default_slot_4(ctx) {
    	let t;

    	return {
    		c() {
    			t = text("LBL File");
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    // (9:2) <Button on:click={loadAny}>
    function create_default_slot_3(ctx) {
    	let t;

    	return {
    		c() {
    			t = text("Any File (Guess format)");
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    // (12:2) <Button on:click={loadAS3}>
    function create_default_slot_2(ctx) {
    	let t;

    	return {
    		c() {
    			t = text("AS3 Array File (Dev)");
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    // (15:2) <Button on:click="{deactivate}">
    function create_default_slot_1(ctx) {
    	let t;

    	return {
    		c() {
    			t = text("Close");
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    // (1:0) <Modal bind:active={active}>
    function create_default_slot(ctx) {
    	let div0;
    	let t1;
    	let div1;
    	let t2;
    	let div2;
    	let t3;
    	let div3;
    	let t4;
    	let div4;
    	let current;

    	const button0 = new Button({
    			props: {
    				$$slots: { default: [create_default_slot_4] },
    				$$scope: { ctx }
    			}
    		});

    	button0.$on("click", /*loadLBL*/ ctx[3]);

    	const button1 = new Button({
    			props: {
    				$$slots: { default: [create_default_slot_3] },
    				$$scope: { ctx }
    			}
    		});

    	button1.$on("click", /*loadAny*/ ctx[4]);

    	const button2 = new Button({
    			props: {
    				$$slots: { default: [create_default_slot_2] },
    				$$scope: { ctx }
    			}
    		});

    	button2.$on("click", /*loadAS3*/ ctx[2]);

    	const button3 = new Button({
    			props: {
    				$$slots: { default: [create_default_slot_1] },
    				$$scope: { ctx }
    			}
    		});

    	button3.$on("click", /*deactivate*/ ctx[1]);

    	return {
    		c() {
    			div0 = element("div");
    			div0.innerHTML = `<h1 class="title svelte-1mamwwo">Import</h1>`;
    			t1 = space();
    			div1 = element("div");
    			create_component(button0.$$.fragment);
    			t2 = space();
    			div2 = element("div");
    			create_component(button1.$$.fragment);
    			t3 = space();
    			div3 = element("div");
    			create_component(button2.$$.fragment);
    			t4 = space();
    			div4 = element("div");
    			create_component(button3.$$.fragment);
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
    		p(ctx, dirty) {
    			const button0_changes = {};

    			if (dirty & /*$$scope*/ 128) {
    				button0_changes.$$scope = { dirty, ctx };
    			}

    			button0.$set(button0_changes);
    			const button1_changes = {};

    			if (dirty & /*$$scope*/ 128) {
    				button1_changes.$$scope = { dirty, ctx };
    			}

    			button1.$set(button1_changes);
    			const button2_changes = {};

    			if (dirty & /*$$scope*/ 128) {
    				button2_changes.$$scope = { dirty, ctx };
    			}

    			button2.$set(button2_changes);
    			const button3_changes = {};

    			if (dirty & /*$$scope*/ 128) {
    				button3_changes.$$scope = { dirty, ctx };
    			}

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
    			if (detaching) detach(div0);
    			if (detaching) detach(t1);
    			if (detaching) detach(div1);
    			destroy_component(button0);
    			if (detaching) detach(t2);
    			if (detaching) detach(div2);
    			destroy_component(button1);
    			if (detaching) detach(t3);
    			if (detaching) detach(div3);
    			destroy_component(button2);
    			if (detaching) detach(t4);
    			if (detaching) detach(div4);
    			destroy_component(button3);
    		}
    	};
    }

    function create_fragment$2(ctx) {
    	let updating_active;
    	let current;

    	function modal_active_binding(value) {
    		/*modal_active_binding*/ ctx[6].call(null, value);
    	}

    	let modal_props = {
    		$$slots: { default: [create_default_slot] },
    		$$scope: { ctx }
    	};

    	if (/*active*/ ctx[0] !== void 0) {
    		modal_props.active = /*active*/ ctx[0];
    	}

    	const modal = new Modal({ props: modal_props });
    	binding_callbacks.push(() => bind(modal, "active", modal_active_binding));

    	return {
    		c() {
    			create_component(modal.$$.fragment);
    		},
    		m(target, anchor) {
    			mount_component(modal, target, anchor);
    			current = true;
    		},
    		p(ctx, [dirty]) {
    			const modal_changes = {};

    			if (dirty & /*$$scope*/ 128) {
    				modal_changes.$$scope = { dirty, ctx };
    			}

    			if (!updating_active && dirty & /*active*/ 1) {
    				updating_active = true;
    				modal_changes.active = /*active*/ ctx[0];
    				add_flush_callback(() => updating_active = false);
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

    async function getFilename() {
    	let filename = await window.dialog.showOpenDialog();

    	if (!filename) {
    		throw "No Dialog Data";
    	}

    	let data = await readFile(filename[0], "utf8");
    	return data;
    }

    function readFile(path, encoding) {
    	return new Promise((resolve, reject) => {
    			window.fs.readFile(path, encoding, function (err, data) {
    				if (err) {
    					reject(err);
    				} else {
    					resolve(data);
    				}
    			});
    		});
    }

    function instance$2($$self, $$props, $$invalidate) {
    	let { active = false } = $$props;

    	function activate() {
    		$$invalidate(0, active = true);
    	}

    	function deactivate() {
    		$$invalidate(0, active = false);
    	}

    	function loadAS3() {
    		getFilename().then(data => {
    			window.level.importAS3(data);
    			deactivate();
    		}).catch(e => {
    			throw e;
    		});
    	}

    	function loadLBL() {
    		getFilename().then(data => {
    			window.level.importLBL(data);
    			deactivate();
    		}).catch(e => {
    			throw e;
    		});
    	}

    	function loadAny() {
    		getFilename().then(data => {
    			window.level.import(data);
    			deactivate();
    		}).catch(e => {
    			throw e;
    		});
    	}

    	function modal_active_binding(value) {
    		active = value;
    		$$invalidate(0, active);
    	}

    	$$self.$set = $$props => {
    		if ("active" in $$props) $$invalidate(0, active = $$props.active);
    	};

    	return [active, deactivate, loadAS3, loadLBL, loadAny, activate, modal_active_binding];
    }

    class ImportModal extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-1mamwwo-style")) add_css$2();
    		init(this, options, instance$2, create_fragment$2, safe_not_equal, { active: 0, activate: 5, deactivate: 1 });
    	}

    	get activate() {
    		return this.$$.ctx[5];
    	}

    	get deactivate() {
    		return this.$$.ctx[1];
    	}
    }

    /* svelte\src\NoteModal.html generated by Svelte v3.17.3 */

    function add_css$3() {
    	var style = element("style");
    	style.id = "svelte-1mamwwo-style";
    	style.textContent = ".title-wrapper.svelte-1mamwwo{text-align:center;background-color:red;margin:0rem 2rem;border-radius:0.25rem;user-select:none}.title.svelte-1mamwwo{text-align:center;text-decoration:none;font-weight:100}.close.svelte-1mamwwo{right:0.5rem;height:1.5rem;font-size:1rem;position:absolute;bottom:1rem}";
    	append(document.head, style);
    }

    // (6:2) <Button on:click="{deactivate}">
    function create_default_slot_1$1(ctx) {
    	let t;

    	return {
    		c() {
    			t = text("Close");
    		},
    		m(target, anchor) {
    			insert(target, t, anchor);
    		},
    		d(detaching) {
    			if (detaching) detach(t);
    		}
    	};
    }

    // (1:0) <Modal bind:active={active}>
    function create_default_slot$1(ctx) {
    	let div0;
    	let t1;
    	let div1;
    	let current;

    	const button = new Button({
    			props: {
    				$$slots: { default: [create_default_slot_1$1] },
    				$$scope: { ctx }
    			}
    		});

    	button.$on("click", /*deactivate*/ ctx[1]);

    	return {
    		c() {
    			div0 = element("div");
    			div0.innerHTML = `<h1 class="title svelte-1mamwwo">Note Content</h1>`;
    			t1 = space();
    			div1 = element("div");
    			create_component(button.$$.fragment);
    			attr(div0, "class", "title-wrapper svelte-1mamwwo");
    			attr(div1, "class", "close svelte-1mamwwo");
    		},
    		m(target, anchor) {
    			insert(target, div0, anchor);
    			insert(target, t1, anchor);
    			insert(target, div1, anchor);
    			mount_component(button, div1, null);
    			current = true;
    		},
    		p(ctx, dirty) {
    			const button_changes = {};

    			if (dirty & /*$$scope*/ 32) {
    				button_changes.$$scope = { dirty, ctx };
    			}

    			button.$set(button_changes);
    		},
    		i(local) {
    			if (current) return;
    			transition_in(button.$$.fragment, local);
    			current = true;
    		},
    		o(local) {
    			transition_out(button.$$.fragment, local);
    			current = false;
    		},
    		d(detaching) {
    			if (detaching) detach(div0);
    			if (detaching) detach(t1);
    			if (detaching) detach(div1);
    			destroy_component(button);
    		}
    	};
    }

    function create_fragment$3(ctx) {
    	let updating_active;
    	let current;

    	function modal_active_binding(value) {
    		/*modal_active_binding*/ ctx[4].call(null, value);
    	}

    	let modal_props = {
    		$$slots: { default: [create_default_slot$1] },
    		$$scope: { ctx }
    	};

    	if (/*active*/ ctx[0] !== void 0) {
    		modal_props.active = /*active*/ ctx[0];
    	}

    	const modal = new Modal({ props: modal_props });
    	binding_callbacks.push(() => bind(modal, "active", modal_active_binding));

    	return {
    		c() {
    			create_component(modal.$$.fragment);
    		},
    		m(target, anchor) {
    			mount_component(modal, target, anchor);
    			current = true;
    		},
    		p(ctx, [dirty]) {
    			const modal_changes = {};

    			if (dirty & /*$$scope*/ 32) {
    				modal_changes.$$scope = { dirty, ctx };
    			}

    			if (!updating_active && dirty & /*active*/ 1) {
    				updating_active = true;
    				modal_changes.active = /*active*/ ctx[0];
    				add_flush_callback(() => updating_active = false);
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

    function instance$3($$self, $$props, $$invalidate) {
    	let { active = false } = $$props;
    	let { content = "" } = $$props;

    	function activate() {
    		$$invalidate(0, active = true);
    	}

    	function deactivate() {
    		$$invalidate(0, active = false);
    	}

    	function modal_active_binding(value) {
    		active = value;
    		$$invalidate(0, active);
    	}

    	$$self.$set = $$props => {
    		if ("active" in $$props) $$invalidate(0, active = $$props.active);
    		if ("content" in $$props) $$invalidate(2, content = $$props.content);
    	};

    	return [active, deactivate, content, activate, modal_active_binding];
    }

    class NoteModal extends SvelteComponent {
    	constructor(options) {
    		super();
    		if (!document.getElementById("svelte-1mamwwo-style")) add_css$3();

    		init(this, options, instance$3, create_fragment$3, safe_not_equal, {
    			active: 0,
    			content: 2,
    			activate: 3,
    			deactivate: 1
    		});
    	}

    	get activate() {
    		return this.$$.ctx[3];
    	}

    	get deactivate() {
    		return this.$$.ctx[1];
    	}
    }

    exports.Button = Button;
    exports.ImportModal = ImportModal;
    exports.NoteModal = NoteModal;

    return exports;

}({}));
