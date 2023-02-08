document.addEventListener('readystatechange', () => {
    if (document.readyState === "interactive") {
        onDOMFinished();
    }
});

function onDOMFinished() {
    setupGallery();
    setupToolTips();
}

function showPreview(gimp) {
    const previewDiv = document.querySelector("#preview");
    if (previewDiv.children.length === 0) {
        let previewImg = new Image();
        previewImg.src = gimp.dataset.fullurl;
        previewDiv.appendChild(previewImg);
        previewDiv.style.display = "block";
    } else {
        previewDiv.style.display = "none";
        previewDiv.innerHTML = "";
    }

}

function setupGallery() {
    const allGalleries = document.querySelectorAll(".gallery");
    const allGalleryImages = document.querySelectorAll(".gallery img");

    const resizeAllGalleries = function () {
        allGalleries.forEach(gallery => {
            resizeGalleryItems(gallery)
        });
    }

    const onGalleryImageLoaded = function (gimg) {
        gimg.style.display = "inline";
        gimg.addEventListener("click", function (_event) {
            showPreview(gimg);
        });
        resizeAllGalleries();
    }

    allGalleryImages.forEach(gimg => {
        if (gimg.complete) {
            onGalleryImageLoaded(gimg);
        } else {
            gimg.addEventListener("load", function (_event) {
                onGalleryImageLoaded(gimg);
            });
            gimg.addEventListener('error', function (err) {
                console.log(err);
            });
        }

    });

    window.addEventListener("resize", function (_event) {
        resizeAllGalleries();
    });
}

function resizeGalleryItems(gallery) {
    let children = gallery.querySelectorAll("img");

    children.forEach(child => {
        console.assert(gallery.classList.contains("gallery"))
        let computedGalleryStyle = window.getComputedStyle(gallery);
        let rowHeight = parseInt(computedGalleryStyle.getPropertyValue('grid-auto-rows'));
        let rowGap = parseInt(computedGalleryStyle.getPropertyValue('grid-row-gap'));
        let rowSpan = Math.ceil((child.getBoundingClientRect().height + rowGap) / (rowHeight + rowGap));
        child.style.gridRowEnd = "span " + rowSpan;
    });
}

function setupToolTips() {
    const tooltip = document.querySelector("#tooltip");

    function onmm(e) {
        let parentContent = e.currentTarget;
        let newX = e.clientX + 10;
        let newY = e.clientY + 10;
        tooltip.style.top = newY + 'px'
        tooltip.style.left = newX + 'px'
        tooltip.style.display = "block";
        tooltip.innerText = parentContent.dataset.disc;
    }

    const thingsThatNeedToolTip = document.querySelectorAll(".gallery img");
    thingsThatNeedToolTip.forEach(contentObject => {
        if (contentObject.dataset.disc.trim() !== "") {
            contentObject.addEventListener("mousemove", onmm, false);
        }
        contentObject.addEventListener("mouseleave", _e => {
            tooltip.style.display = "none"
        }, false);
    });
}