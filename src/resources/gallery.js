function init() {
    setupGalleryResizer();
    setupToolTips();
}

function setupGalleryResizer() {
    const getVal = function (elem, style) {
        return parseInt(window.getComputedStyle(elem).getPropertyValue(style));
    };
    const getHeight = function (item) {
        return item.querySelector('.content').getBoundingClientRect().height;
    };

    const galleries = document.querySelectorAll('.gallery');
    galleries.forEach(gallery => {
        gallery.querySelectorAll('.gallery-item').forEach(function (item) {
            item.style.opacity = "0"; // Hide all the pictures while they are loading
            item.addEventListener('click', function () {
                item.classList.toggle('full');
            });
        });
    });


    const resizeAll = function () {
        galleries.forEach(gallery => {
            const altura = getVal(gallery, 'grid-auto-rows');
            const gap = getVal(gallery, 'grid-row-gap');
            gallery.querySelectorAll('.gallery-item').forEach(function (item) {
                item.style.gridRowEnd = "span " + Math.ceil((getHeight(item) + gap) / (altura + gap));
                item.style.opacity = "1"; // Once they are loaded show them again
            });
        });
    };
    window.addEventListener('resize', resizeAll);
    resizeAll();
}

function setupToolTips() {
    const tooltip = document.querySelector(".tooltip");

    function onmm(e) {
        let parentContent = e.currentTarget;
        let newX = e.clientX + 10;
        let newY = e.clientY + 10;
        tooltip.style.top = newY + 'px'
        tooltip.style.left = newX + 'px'
        tooltip.style.display = "block";
        tooltip.innerText = parentContent.dataset.disc;
    }

    const thingsThatNeedToolTip = document.querySelectorAll(".content");
    thingsThatNeedToolTip.forEach(contentObject => {
        if (contentObject.dataset.disc.trim() !== "") {
            contentObject.addEventListener("mousemove", onmm, false);
        }
        contentObject.addEventListener("mouseleave", e => {
            tooltip.style.display = "none"
        }, false);
    });
}