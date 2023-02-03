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
    const allGalleryImages = document.querySelectorAll(".gallery img");

    const onGalleryImageLoaded = function (gimg) {
        gimg.addEventListener("click", function (event) {
            showPreview(gimg);
        });
        resizeGridItem(gimg);
    }

    allGalleryImages.forEach(gimg => {
        if (gimg.complete) {
            onGalleryImageLoaded(gimg);
        } else {
            gimg.addEventListener("load", function (event) {
                onGalleryImageLoaded(event.target);
            });
            gimg.addEventListener('error', function (err) {
                console.log(err);
            });
        }

    });

    window.addEventListener("resize", function (event) {
        allGalleryImages.forEach(resizeGridItem);
    });
}

function resizeGridItem(item) {
    let gallery = item.parentElement;
    console.assert(gallery.classList.contains("gallery"))
    let computedGalleryStyle = window.getComputedStyle(gallery);
    let rowHeight = parseInt(computedGalleryStyle.getPropertyValue('grid-auto-rows'));
    let rowGap = parseInt(computedGalleryStyle.getPropertyValue('grid-row-gap'));
    let rowSpan = Math.ceil((item.getBoundingClientRect().height + rowGap) / (rowHeight + rowGap));
    item.style.gridRowEnd = "span " + rowSpan;
    item.style.display = "inline";
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
        contentObject.addEventListener("mouseleave", e => {
            tooltip.style.display = "none"
        }, false);
    });
}