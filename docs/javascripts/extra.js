// Galois Documentation - Scripts personnalisés

// Feedback sur la documentation
document.addEventListener("DOMContentLoaded", function() {
  // Animation au scroll
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('visible');
      }
    });
  }, { threshold: 0.1 });

  document.querySelectorAll('.grid.cards > ul > li').forEach(card => {
    observer.observe(card);
  });
});

// Copie du code avec feedback
document.addEventListener("DOMContentLoaded", function() {
  document.querySelectorAll('.md-clipboard').forEach(btn => {
    btn.addEventListener('click', function() {
      const originalTitle = this.title;
      this.title = 'Copié !';
      setTimeout(() => {
        this.title = originalTitle;
      }, 2000);
    });
  });
});

// Smooth scroll pour les ancres
document.addEventListener("DOMContentLoaded", function() {
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function(e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });
      }
    });
  });
});

// Version actuelle
console.log('📚 Documentation Galois v0.2.0');
console.log('🔗 https://github.com/TataneSan/galois');
