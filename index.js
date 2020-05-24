const rust = import('./pkg/index')
const canvas = document.getElementById("rustCanvas");
const gl = canvas.getContext('webgl', {antialiased : true});


rust.then(m => {
    if(!gl)
    {
        alert('Failed to initialize webgl');
        return;
    }

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA,gl.ONE_MINUS_SRC_ALPHA);


    const gameClient = new m.GameClient();
    const initialTime = Date.now();

    function render()
    {
        window.requestAnimationFrame(render);
        const currTime = Date.now();

        if(window.innerHeight != canvas.height || window.innerWidth != canvas.width)
        {
            canvas.height = window.innerHeight;
            canvas.clientHeight= window.innerHeight;
            canvas.style.height= window.innerHeight;

            canvas.width= window.innerWidth;
            canvas.clientWidth= window.innerWidth;
            canvas.style.width= window.innerWidth;

            gl.viewport(0,0,window.innerWidth, window.innerHeight);
        }

        let elapsedTime = currTime - initialTime;
        gameClient.update(elapsedTime, window.innerHeight,window.innerHeight);
        gameClient.render();


        //Rust update call

        //Rust render call


    }

    //kicking the loop
    render();

})
