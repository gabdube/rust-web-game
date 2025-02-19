import * as engine_mod from "./engine.js";

let boundedRun = null;

function show(element) {
    element.classList.remove('hidden');
}

function show_critical_error(error) {
    const panel = document.getElementById("errorPanel");
    const error_message = panel.children[2];
    const error_traceback = panel.lastElementChild;

    error_message.textContent = error.msg;

    if (error.traceback) {
        show(document.getElementById("errorDetails"));
        error_traceback.textContent = error.traceback.toString();
    }

    show(panel);
}

function run(engine, time) {
    engine_mod.update(engine);
    engine_mod.render(engine);

    if (engine.exit) {
        return;
    }

    if (engine.reload) {
        engine_mod.reload(engine)
            .then(() => requestAnimationFrame(boundedRun) );
    } else {
        requestAnimationFrame(boundedRun);
    }
}

async function init_app() {
    const engine = await engine_mod.init();
    if (!engine) {
        show_critical_error(engine_mod.get_last_error());
        return;
    }

    boundedRun = run.bind(null, engine);
    boundedRun();
}

init_app();
